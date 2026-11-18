"""On the device: the Rust keyboard draws its keys when it is shown again soon after hiding.

python3 -I scripts/render-benchmark/reshow.py OUTPUT [PATH ...], from the repository, with
ANDROID_SERIAL (the dedicated emulator) and AGENT_DEVICE (the exact device_open command) set.
PATH is any of self, back, outside, switch (default: all four):

  self     the keyboard's own Hide keyboard action (requestHideSelf); the field is tapped again
  back     the Back key; the field is tapped again
  outside  a tap outside the field (the probe hides the keyboard, as many apps do); then the field
  switch   another app comes to the front, then the probe does again

Each path is tried at gaps of 100, 300, 600 and 1000 ms between hiding and showing. The hide and
the show go in one adb shell call, so the gap is not stretched by adb's own start-up. Each trial
starts from a keyboard hidden for 1.5 s and shown again, its keys checked drawn, so a blank one
left by the trial before does not carry over. Those calls and the scroll down the action overflow
to Hide keyboard are where these events bypass agent-device: its gesture helper holds a UI
automation session, and while one is open Android unbinds the probe's accessibility service, which
the check reads the keyboard through. Three seconds after each show the check needs, since the
hide: a native window released and a new one acquired, then a frame drawn on it; and the
keyboard's key rows matching a settled screenshot taken before any hiding (or, while the action
overflow is still open, a picture of it). A keyboard that draws only its background differs from
that in most of those pixels. A hide too quick for Android to release the window leaves the keys
drawn: that trial passes if they still match, and prints as kept. Coordinates are for the
1080x2400 portrait AVD. Prints numbers only; saves the screenshots and a results.json. OUTPUT must
not exist yet.
"""
import json
import os
import re
import runpy
import subprocess
import sys
import time
from pathlib import Path

here = Path(__file__)
api = runpy.run_path(str(here.with_name("a11y_device.py")))
pixels = runpy.run_path(str(here.with_name("dirty-vs-full.py")))["pixels"]
PROBE = api["PROBE"]
PKG = api["PKG"]
GAPS = (100, 300, 600, 1000)
FIELD = (540, 480)
# An empty part of the probe, below its buttons: the title above the field takes its own taps.
OUTSIDE = (540, 1100)
# The keys, below the suggestion strip and above the navigation bar, on the 1080x2400 AVD.
KEY_ROWS = range(1700, 2250)
# The share of key-row pixels a frame without keys still has right: well above any change a
# drawn keyboard shows between two screenshots, well below a blank one.
MATCH = 0.98


def hide_and_show(path, gap, hide_key):
    """The shell commands that hide the keyboard, wait `gap` ms, and show it again."""
    pause = "sleep %.3f" % (gap / 1000)
    tap = "cmd input tap %d %d" % FIELD
    if path == "self":
        return "cmd input tap %d %d; %s; %s" % (*hide_key, pause, tap)
    if path == "back":
        return "cmd input keyevent 4; %s; %s" % (pause, tap)
    if path == "outside":
        return "cmd input tap %d %d; %s; %s" % (*OUTSIDE, pause, tap)
    if path == "switch":
        home = "cmd activity start-activity -a android.intent.action.MAIN -c android.intent.category.HOME"
        back = "cmd activity start-activity -f 0x10200000 -n %s/.ProbeActivity" % PROBE
        return "%s; %s; %s; sleep 0.4; %s" % (home, pause, back, tap)
    raise ValueError(path)


def key_rows_match(reference, shot):
    _, _, bpp, a = reference
    _, _, _, b = shot
    total = same = 0
    for y in KEY_ROWS:
        for x in range(0, len(a[y]), bpp * 4):
            total += 1
            same += a[y][x:x + 3] == b[y][x:x + 3]
    return same / total


def native_events(log):
    """Numeric aw-render window events, in order: ("released"|"acquired"|"frame", generation)."""
    events = []
    for line in log.splitlines():
        if m := re.search(r"window (released|acquired) generation=(\d+)", line):
            events.append((m.group(1), int(m.group(2))))
        elif " frame draw_ns=" in line:
            events.append(("frame", None))
    return events


def drew_new_window(events):
    released = [i for i, (kind, _) in enumerate(events) if kind == "released"]
    acquired = [i for i, (kind, _) in enumerate(events) if kind == "acquired"]
    if not released or not acquired or acquired[-1] < released[0]:
        return False
    return any(kind == "frame" for kind, _ in events[acquired[-1] + 1:])


def main():
    out = Path(sys.argv[1])
    if out.exists():
        raise SystemExit("refusing to overwrite " + str(out))
    paths = sys.argv[2:] or ["self", "back", "outside", "switch"]
    agent = os.environ["AGENT_DEVICE"]
    device = api["Device"](out)
    original = device.original_accessibility()
    results = []
    try:
        device.mute()
        for tag in ("aw-render", "aw-profile"):
            device.shell("setprop", "log.tag." + tag, "DEBUG")
        device.renderer("rust")
        device.probe(extras=("--ez", "outsideHides", "true"))
        # node() finds the keyboard's actions through the probe's accessibility service.
        device.services(api["SERVICE"])
        time.sleep(2.0)
        reference = out.resolve() / "reference.png"
        subprocess.check_output([agent, "screenshot", str(reference)], text=True)
        settled = pixels(reference)
        for path in paths:
            for gap in GAPS:
                results.append(trial(device, agent, settled, path, gap, out))
                r = results[-1]
                status = "BLANK" if not r["ok"] else "ok" if r["window_released"] else "kept"
                print("%-8s %5d ms  window=%d  keys=%.3f  %s" % (
                    path, gap, r["new_window_drawn"], r["key_rows_matching"], status))
    finally:
        device.restore_accessibility(original)
        for tag in ("aw-render", "aw-profile"):
            device.shell("setprop", "log.tag." + tag, "INFO")
    device.save("results.json", results)
    failed = [r for r in results if not r["ok"]]
    print("%d of %d trials drew their keys" % (len(results) - len(failed), len(results)))
    if failed:
        raise SystemExit(1)


def settle_shown(device):
    shown = lambda: "mInputShown=true" in device.shell("dumpsys", "input_method")
    if not shown():
        device.tap(*FIELD)
    device.wait(shown, "IME not shown")
    time.sleep(1.5)


def start_drawn(device, agent, settled, out):
    """Hide the keyboard long enough to draw again when shown, show it, and check its keys."""
    device.adb("shell", hide_and_show("back", 1500, None))
    time.sleep(1.5)
    settle_shown(device)
    shot = out.resolve() / "start.png"
    subprocess.check_output([agent, "screenshot", str(shot)], text=True)
    if key_rows_match(settled, pixels(shot)) < MATCH:
        raise RuntimeError("the keyboard did not draw its keys after a slow hide and show")


def trial(device, agent, settled, path, gap, out):
    start_drawn(device, agent, settled, out)
    hide_key = None
    if path == "self":
        hide_key = node(device, "Hide keyboard")
        if not hide_key:
            if not node(device, "More actions"):
                # The suggestions are showing in place of the action row: open the row first.
                device.tap(*device.wait(lambda: node(device, "Expand actions"), "No Expand actions button"))
            device.tap(*device.wait(lambda: node(device, "More actions"), "No More actions button"))
            time.sleep(1.0)
            for _ in range(4):
                # The action can sit below the overflow's first screen: scroll the grid up to it.
                hide_key = node(device, "Hide keyboard")
                if hide_key:
                    break
                device.adb("shell", "input swipe 540 2150 540 1800 400")
                time.sleep(1.0)
            hide_key = device.wait(lambda: node(device, "Hide keyboard"), "No Hide keyboard action")
            time.sleep(1.0)
    overflow = None
    if node(device, "Hide keyboard"):
        # The action overflow is open in place of the keys: keep a picture of it as drawn now.
        before = out.resolve() / ("%s-%d-before.png" % (path, gap))
        subprocess.check_output([agent, "screenshot", str(before)], text=True)
        overflow = pixels(before)
    device.adb("logcat", "-c")
    device.adb("shell", hide_and_show(path, gap, hide_key))
    time.sleep(3.0)
    shown = "mInputShown=true" in device.shell("dumpsys", "input_method")
    log = device.adb("logcat", "-d", "-s", "aw-render:D", "aw-profile:D")
    events = native_events(log)
    shot = out.resolve() / ("%s-%d.png" % (path, gap))
    subprocess.check_output([agent, "screenshot", str(shot)], text=True)
    # Compare with what the keyboard should show now: the overflow if it is still open (the
    # self path opens it, and a keyboard shown again may keep it), otherwise its keys.
    reference = settled
    if node(device, "Hide keyboard"):
        if overflow is None:
            raise RuntimeError("the action overflow opened during a %s trial" % path)
        reference = overflow
    matching = key_rows_match(reference, pixels(shot))
    released = any(kind == "released" for kind, _ in events)
    drawn = drew_new_window(events)
    views = [int(m.group(1)) for m in re.finditer(r"view event=(\d+)", log)]
    return {"path": path, "gap_ms": gap, "shown": shown, "window_released": int(released),
            "new_window_drawn": int(drawn), "key_rows_matching": round(matching, 4),
            "native_events": [k for k, _ in events], "view_events": views,
            "ok": shown and matching >= MATCH and (drawn or not released)}


def node(device, label):
    """The centre of the visible keyboard node with this label, from the probe's accessibility dump."""
    for n in device.tree()["nodes"]:
        if n.get("visible") and label in (n.get("text"), n.get("description")):
            x1, y1, x2, y2 = n["bounds"]
            return (x1 + x2) // 2, (y1 + y2) // 2
    return None


if __name__ == "__main__":
    main()
