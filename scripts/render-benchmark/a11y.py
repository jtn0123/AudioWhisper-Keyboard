"""Fresh IME dumps for both renderers; run with python3 -I via a11y.sh."""
import argparse
import hashlib
from pathlib import Path
import runpy

api = runpy.run_path(str(Path(__file__).with_name("a11y_device.py")))
Device, SERVICE = api["Device"], api["SERVICE"]


def labels(nodes):
    return {n.get("text") for n in nodes}


def capture(device, renderer):
    device.renderer(renderer)
    device.services(SERVICE)
    device.probe(no_suggestions=True)
    device.services(SERVICE)
    prefix = renderer + "-"
    # The probe starts a fresh input session, which may auto-shift its first key.
    tree = device.capture(prefix + "initial", lambda ns: "e" in labels(ns) or "E" in labels(ns))
    if "E" in labels(tree["nodes"]):
        device.tap(70, 2060)
    if device.prefs().get("smartbar__shared_actions_expanded") == "true":
        device.tap(1025, 1618)
    device.capture(prefix + "letters", lambda ns: "e" in labels(ns) and
                   not any(n.get("description") == "Undo" for n in ns))
    device.tap(70, 2060)
    device.capture(prefix + "shifted", lambda ns: "E" in labels(ns))
    # A hold invokes the configured caps lock action without double-tap timing ambiguity.
    device.hold(70, 2060, 1000)
    # Verify persistence across one key commit, then erase it before any tree dump.
    device.tap(275, 1750)
    device.wait(lambda: device.digest()["utf16_units"] == 1, "Caps-lock probe did not commit")
    caps_digest = device.digest()
    if caps_digest["sha256"] != hashlib.sha256(b"E").hexdigest():
        raise RuntimeError("Caps-lock probe was not uppercase")
    device.save(prefix + "caps-probe.json", caps_digest)
    device.tap(1015, 2060)
    device.wait(lambda: device.digest()["utf16_units"] == 0, "Caps-lock probe was not erased")
    device.capture(prefix + "caps-lock", lambda ns: "E" in labels(ns) and
                   (renderer == "compose" or any(n.get("stateDescription") == "Caps lock enabled" for n in ns)))
    device.tap(70, 2060)
    device.capture(prefix + "unshifted", lambda ns: "e" in labels(ns))
    device.tap(70, 2190)
    device.capture(prefix + "symbols", lambda ns: "1" in labels(ns) and "e" not in labels(ns)
                   and "E" not in labels(ns) and (renderer == "compose" or any(n.get("textEntryKey") for n in ns)))
    device.tap(70, 2190)
    device.capture(prefix + "returned", lambda ns: "e" in labels(ns))
    device.reset(no_suggestions=True, field="number")
    device.capture(prefix + "numeric", lambda ns: "1" in labels(ns) and "2" in labels(ns) and "e" not in labels(ns)
                   and (renderer == "compose" or any(n.get("textEntryKey") for n in ns)))
    device.reset(no_suggestions=True)
    device.capture(prefix + "letters-again", lambda ns: "e" in labels(ns))
    if device.prefs().get("smartbar__shared_actions_expanded") != "true":
        device.tap(1025, 1618)
    device.capture(prefix + "expanded", lambda ns: any(n.get("description") == "Undo" for n in ns))
    device.tap(1025, 1618)
    device.capture(prefix + "collapsed", lambda ns: not any(n.get("description") == "Undo" for n in ns))
    # Queue capture during the 700 ms background hold, not after input has lifted.
    request = device.request(delay=450)
    process = device.swipe_background(275, 1750)
    try:
        data = device.wait(lambda: device.response(request), "Popup dump timed out")
        if process.poll() is not None:
            raise RuntimeError("Popup response arrived after the hold ended")
        if "nodes" not in data or not any(n.get("text") in ("é", "è", "ê", "ë") for n in data["nodes"]):
            raise RuntimeError("Popup was not open at capture time")
        device.save(prefix + "popup.json", data)
        print(prefix + "popup", len(data["nodes"]), flush=True)
    finally:
        _, error = process.communicate(timeout=10)
        if process.returncode:
            raise RuntimeError("Background swipe failed: " + error)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output")
    parser.add_argument("--renderer", choices=("compose", "rust"))
    args = parser.parse_args()
    device = Device(args.output)
    original = device.original_accessibility()
    device.save("original-accessibility.json", original)
    capture_prefs = device.capture_preferences()
    try:
        device.mute()
        for renderer in ([args.renderer] if args.renderer else ["compose", "rust"]):
            capture(device, renderer)
    finally:
        device.restore_capture_preferences(capture_prefs)
        device.restore_accessibility(original)
        device.save("restored-accessibility.json", device.original_accessibility())


if __name__ == "__main__":
    main()
