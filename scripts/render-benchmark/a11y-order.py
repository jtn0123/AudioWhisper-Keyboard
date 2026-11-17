"""The Rust keyboard's reading order as Android exposes it, for `compare-a11y.py --order`.

Saves only each node's parent, class, bounds, text-entry flag and traversal hints: never a label,
description, state text or word. Labels are read in memory only, to find the keys to press. The
row of words is captured after typing two letters, so that tree is redacted on the device too and
the letters are erased before the next capture. Run on the dedicated runtime serial:

    ANDROID_SERIAL=<runtime> AGENT_DEVICE=<agent-device command> \\
        python3 -I scripts/render-benchmark/a11y-order.py build/a11y-order-run
"""
import argparse
import os
from pathlib import Path
import runpy
import subprocess
import time
import uuid

api = runpy.run_path(str(Path(__file__).with_name("a11y_device.py")))
Device, SERVICE, PROBE = api["Device"], api["SERVICE"], api["PROBE"]
FIELDS = ("parent", "class", "bounds", "textEntryKey", "visible", "traversalAfter", "traversalBefore")
KEY = "android.inputmethodservice.Keyboard$Key"
ARROW, SHIFT, DELETE, E = (1025, 1618), (70, 2060), (1015, 2060), (275, 1750)


def dump(device, redact=False, delay=0):
    request = uuid.uuid4().hex
    device.shell("am", "broadcast", "-a", PROBE + ".IME", "-p", PROBE, "--es", "request", request,
                 "--ei", "delay", delay, "--ez", "redact", str(redact).lower())
    data = device.wait(lambda: device.response(request), "Dump service did not respond", timeout=20)
    if "nodes" not in data:
        raise RuntimeError("No tree: " + data.get("error", "unknown"))
    return data["nodes"]


def save(device, name, nodes):
    device.save(name + ".json", {"nodes": [{k: n[k] for k in FIELDS if k in n} for n in nodes]})
    print(name, len(nodes), flush=True)


def settle(device, name, ready, redact=False):
    """Wait until the tree satisfies `ready` and holds still for 400 ms, then save its geometry."""
    previous, since = None, time.monotonic()

    def check():
        nonlocal previous, since
        nodes = dump(device, redact)
        valid = ready(nodes)
        if nodes != previous or not valid:
            previous, since = nodes, time.monotonic()
        return nodes if valid and time.monotonic() - since >= .4 else None
    nodes = device.wait(check, "Expected stable IME state not reached: " + name, timeout=30)
    save(device, name, nodes)
    return nodes


def shown(device):
    """The tree once the letter keys are up, or None while the keyboard window reopens."""
    try:
        nodes = dump(device)
    except RuntimeError:
        return None
    return nodes if {"e", "E"} & labels(nodes) else None


def labels(nodes):
    return {n.get("text") for n in nodes}


def middle(node):
    b = node["bounds"]
    return (b[0] + b[2]) // 2, (b[1] + b[3]) // 2


def words(nodes):
    """A word slot is wider than the row is tall; the Smartbar's buttons are square or narrower."""
    keys = [n for n in nodes if n["class"] == KEY]
    if not keys:
        return False
    top = min(k["bounds"][1] for k in keys)
    return any(n["parent"] == keys[0]["parent"] and not n["textEntryKey"] and n["bounds"][3] <= top
               and n["bounds"][2] - n["bounds"][0] > 1.5 * (n["bounds"][3] - n["bounds"][1])
               for n in nodes)


def erase(device):
    for _ in range(8):
        if device.digest()["utf16_units"] == 0:
            return
        device.tap(*DELETE)
        time.sleep(.3)
    device.wait(lambda: device.digest()["utf16_units"] == 0, "Typed letters were not erased")


def hold(x, y, duration):
    """Hold with shell input even under AGENT_DEVICE: while agent-device holds a press, the probe
    service answers no requests (2026-11-17, aw_keyboard_ssd), so the popup could not be read."""
    command = ["adb", "-s", os.environ["ANDROID_SERIAL"], "shell", "input", "swipe",
               str(x), str(y), str(x), str(y), str(duration)]
    return subprocess.Popen(command, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)


def capture(device):
    device.renderer("rust")
    device.services(SERVICE)
    device.probe(no_suggestions=True)
    device.services(SERVICE)
    first = device.wait(lambda: shown(device), "Letters did not appear", timeout=20)
    if "E" in labels(first):
        device.tap(*SHIFT)
    if device.prefs().get("smartbar__shared_actions_expanded") == "true":
        device.tap(*ARROW)
    letters = settle(device, "letters", lambda ns: "e" in labels(ns) and
                     not any(n.get("description") == "Undo" for n in ns))
    device.tap(*ARROW)
    settle(device, "expanded", lambda ns: any(n.get("description") == "Undo" for n in ns))
    device.tap(*ARROW)
    settle(device, "collapsed", lambda ns: "e" in labels(ns) and
           not any(n.get("description") == "Undo" for n in ns))
    # Two letters bring up the row of words. Only a redacted tree is taken while they are typed.
    at = {n.get("text"): middle(n) for n in letters if n["class"] == KEY}
    device.reset(no_suggestions=False)
    for letter in "th":
        device.tap(*at[letter])
    device.wait(lambda: device.digest()["utf16_units"] == 2, "Two letters were not typed")
    settle(device, "words", words, redact=True)
    erase(device)
    device.reset(no_suggestions=True)
    # A held key's popup: capture while the hold lasts, then erase whatever the lift chose. The
    # reset reopens the keyboard, so wait for its keys first or the hold lands on nothing.
    device.wait(lambda: shown(device), "Letters did not return", timeout=20)
    process = hold(*E, 3000)
    try:
        nodes = device.wait(lambda: (lambda ns: ns if any(n["textEntryKey"] and n["class"] != KEY
                                                          for n in ns) else None)(dump(device)),
                            "Popup did not open", timeout=20)
        if process.poll() is not None:
            raise RuntimeError("Popup captured after the hold ended")
        save(device, "popup", nodes)
    finally:
        _, error = process.communicate(timeout=30)
        if process.returncode:
            raise RuntimeError("Hold failed: " + error)
    erase(device)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output")
    args = parser.parse_args()
    device = Device(args.output)
    original = device.original_accessibility()
    device.save("original-accessibility.json", original)
    try:
        device.mute()
        capture(device)
    finally:
        device.restore_accessibility(original)
        device.save("restored-accessibility.json", device.original_accessibility())


if __name__ == "__main__":
    main()
