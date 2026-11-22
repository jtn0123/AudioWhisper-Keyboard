"""Layout rebuild timing on the dedicated emulator, with the Rust renderer.

Switches the keyboard's mode (?123, then ABC) and its subtype (the language key) by tapping
through agent-device, and times each switch from the tap's last touch to the last frame the
renderer posted for it: the new keyboard's rebuild and its first draw. Needs ANDROID_SERIAL,
AGENT_DEVICE (the exact device_open command), the probe's empty text field focused on the
letters, and two subtypes with the language key showing. Saves only numbers. Run with
python3 -I.
"""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import time

FRAME = re.compile(r"frame draw_ns=(\d+) post_ns=(\d+) touch_ns=(\d+)")
# The bottom row's first key (?123 on the letters, ABC on the symbols) and the language key.
KEYS = {"mode": (89, 2198), "subtype": (335, 2198)}
# Touches closer together than this belong to one tap.
TAP_NS = 400_000_000


def adb(*args):
    serial = os.environ["ANDROID_SERIAL"]
    if serial == "emulator-5554":
        raise ValueError("Reserved emulator")
    return subprocess.run(["adb", "-s", serial, *args], check=True, capture_output=True, text=True,
                          timeout=150).stdout


def press(x, y):
    subprocess.run([os.environ["AGENT_DEVICE"], "press", str(x), str(y)], check=True, capture_output=True,
                   timeout=120)


def frames():
    lines = adb("logcat", "-d", "-s", "aw-render:D").splitlines()
    return [tuple(map(int, m.groups())) for m in map(FRAME.search, lines) if m]


def taps(rows):
    """Per tap: the last post after its last touch, in ms, and how many frames it posted."""
    found = []
    for _, post, touch in rows:
        if not touch:
            continue
        tap = next((t for t in found if abs(touch - t["first"]) < TAP_NS), None)
        if tap is None:
            tap = {"first": touch, "touch": touch, "post": post, "frames": 0}
            found.append(tap)
        tap["touch"], tap["post"] = max(tap["touch"], touch), max(tap["post"], post)
        tap["frames"] += 1
    return [{"ms": (t["post"] - t["touch"]) / 1e6, "frames": t["frames"]} for t in found]


def percentile(values, p):
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, round(p / 100 * (len(ordered) - 1)))] if ordered else None


def phase(name, count, pause):
    adb("logcat", "-c")
    for _ in range(count):
        press(*KEYS[name])
        time.sleep(pause)
    time.sleep(2)
    found = taps(frames())
    ms = [t["ms"] for t in found]
    return {"switches": count, "taps_seen": len(found), "ms": ms, "frames": [t["frames"] for t in found],
            "p50": percentile(ms, 50), "p95": percentile(ms, 95), "max": max(ms, default=None)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--switches", type=int, default=20, help="an even number, so both end where they began")
    parser.add_argument("--pause", type=float, default=1.5)
    args = parser.parse_args()
    assert args.switches % 2 == 0, "an even number of switches"
    args.output.mkdir(parents=True, exist_ok=False)
    adb("shell", "setprop", "log.tag.aw-render", "DEBUG")
    try:
        result = {name: phase(name, args.switches, args.pause) for name in KEYS}
    finally:
        adb("shell", "setprop", "log.tag.aw-render", "INFO")
    (args.output / "rebuild.json").write_text(json.dumps(result, indent=2) + "\n")
    for name, numbers in result.items():
        print(f"{name}: {numbers['taps_seen']}/{numbers['switches']} taps, p50 {numbers['p50']:.1f} ms, "
              f"p95 {numbers['p95']:.1f} ms, max {numbers['max']:.1f} ms", flush=True)


if __name__ == "__main__":
    main()
