"""Where a switch's tap waits before the keyboard sees it, from a system trace, without root.

Taps the mode key and the language key 10 times each, as switch-cost.py does, with atrace
recording input, view and graphics meanwhile. Each tap is agent-device's press: a DOWN, then an
UP stamped HOLD_MS later. For each kind of switch and each action, medians in ms of:

* stamp to injected: the event's own time to system_server's interceptMotionBeforeQueueing,
  which is where an injected event arrives;
* injected to published: to InputDispatcher's publishMotionEvent on the keyboard's channel;
* published to dispatched: to the keyboard's main thread dispatching it;
* and, for the UP, dispatched to the app vsync its frame ran on (SurfaceFlinger's VSYNC-app tick
  at or before the keyboard's next Choreographer#doFrame), and that tick to the doFrame.

The stamps are the touch times in the renderer's frame log (the first of each tap is its DOWN's);
atrace's clock is the same on an emulator that has not slept. Needs ANDROID_SERIAL, AGENT_DEVICE,
the probe's empty field focused and two subtypes with the language key showing. Keeps the trace
in the output folder and prints and saves only numbers. Run with python3 -I.
"""
import argparse
import bisect
import json
from pathlib import Path
import re
import runpy
import statistics
import time

cost = runpy.run_path(str(Path(__file__).with_name("switch-cost.py")))
adb, press, KEYS, TAP_NS = cost["adb"], cost["press"], cost["KEYS"], cost["TAP_NS"]
HOLD_NS = cost["timing"]["HOLD_MS"] * 1_000_000
PKG = "com.jtn0123.audiowhisper.keyboard"
KINDS, TAPS = ("mode", "subtype"), 10
DEVICE_TRACE = "/data/local/tmp/aw-switch.atrace"
# A slice's start: the task's thread id, the time in seconds, and the slice's name.
SLICE = re.compile(r"^\s*.+?-(\d+)\s+(?:\(\s*[\d-]+\)\s+)?\[\d+\]\s+\S+\s+(\d+\.\d+): tracing_mark_write: B\|\d+\|(.*)$")
VSYNC = re.compile(r"\s(\d+\.\d+): tracing_mark_write: C\|\d+\|VSYNC-app\|")
TOUCH = re.compile(r"frame draw_ns=\d+ post_ns=\d+ touch_ns=(\d+)")
# How far before a DOWN's stamp its events are looked for.
EARLY_NS = 20_000_000


def record(output):
    """Taps each kind of switch with atrace running; the keyboard's pid and the frames' touch times."""
    pid = int(adb("shell", "pidof", PKG).split()[0])
    adb("shell", "setprop", "log.tag.aw-render", "DEBUG")
    adb("logcat", "-c")
    adb("shell", "atrace", "--async_start", "-b", "65536", "-a", PKG, "gfx", "input", "view")
    try:
        time.sleep(1)
        for kind in KINDS:
            for _ in range(TAPS):
                press(*KEYS[kind])
                time.sleep(1.5)
        # Read before stopping: the stop makes every process log, which pushes the frames out.
        log = adb("logcat", "-d", "-s", "aw-render:D")
    finally:
        adb("shell", "atrace", "--async_stop", "-o", DEVICE_TRACE)
        adb("shell", "setprop", "log.tag.aw-render", "INFO")
    adb("pull", DEVICE_TRACE, str(output / "switch.atrace"))
    adb("shell", "rm", "-f", DEVICE_TRACE)
    return pid, sorted(t for t in (int(m.group(1)) for m in map(TOUCH.search, log.splitlines()) if m) if t)


def downs(touches):
    """Each tap's DOWN stamp: the first touch time of each group of touches."""
    found = []
    for touch in touches:
        if not found or touch - found[-1] >= TAP_NS:
            found.append(touch)
    return found


def mark(keyboard, name):
    """Which of the times this measures a slice's start is, if any."""
    if name.startswith("interceptMotionBeforeQueueing"):
        return "injected"
    action = re.search(r"action=(DOWN|UP)\)", name)
    if name.startswith("publishMotionEvent(") and "InputMethod" in name and action:
        return "published " + action.group(1)
    if not keyboard:
        return None
    action = re.match(r"dispatchInputEvent MotionEvent ACTION_(DOWN|UP)\b", name)
    if action:
        return "dispatched " + action.group(1)
    return "frame" if name.startswith("Choreographer#doFrame") else None


def marks(path, pid):
    """The start times, in ns, of each kind of slice measured."""
    found = {"injected": [], "frame": [], "vsync": []}
    with open(path, errors="replace") as trace:
        for line in trace:
            tick = VSYNC.search(line)
            if tick:
                found["vsync"].append(round(float(tick.group(1)) * 1e9))
            match = SLICE.match(line)
            key = match and mark(int(match.group(1)) == pid, match.group(3))
            if key:
                found.setdefault(key, []).append(round(float(match.group(2)) * 1e9))
    return {key: sorted(times) for key, times in found.items()}


def after(times, t):
    i = bisect.bisect_left(times, t)
    return times[i] if i < len(times) else None


def before(times, t):
    i = bisect.bisect_right(times, t) - 1
    return times[i] if i >= 0 else None


def waits(found, taps):
    """Per measure, each tap's time in ms."""
    out = {}
    for down in taps:
        for action, stamp in (("DOWN", down), ("UP", down + HOLD_NS)):
            published = after(found.get("published " + action, []), down - EARLY_NS)
            dispatched = published and after(found.get("dispatched " + action, []), published)
            injected = published and before(found["injected"], published)
            if not (published and dispatched and injected):
                continue
            for name, a, b in (("stamp to injected", stamp, injected), ("injected to published", injected, published),
                               ("published to dispatched", published, dispatched)):
                out.setdefault(f"{action} {name}", []).append((b - a) / 1e6)
            frame = after(found["frame"], dispatched) if action == "UP" else None
            vsync = frame and before(found["vsync"], frame)
            if vsync and vsync >= dispatched:
                out.setdefault("UP dispatched to its frame's vsync", []).append((vsync - dispatched) / 1e6)
                out.setdefault("vsync to the doFrame", []).append((frame - vsync) / 1e6)
    return out


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    pid, touches = record(args.output)
    taps = downs(touches)
    assert len(taps) == TAPS * len(KINDS), f"{len(taps)} taps seen"
    found = marks(args.output / "switch.atrace", pid)
    result = {}
    for i, kind in enumerate(KINDS):
        times = waits(found, taps[i * TAPS:(i + 1) * TAPS])
        result[kind] = {name: {"median": statistics.median(v), "min": min(v), "max": max(v), "n": len(v)}
                        for name, v in times.items()}
        for name, row in result[kind].items():
            print(f"{kind} {name}: median {row['median']:.1f} ms, min {row['min']:.1f}, max {row['max']:.1f}"
                  f" (n={row['n']})", flush=True)
    (args.output / "trace.json").write_text(json.dumps(result, indent=1) + "\n")


if __name__ == "__main__":
    main()
