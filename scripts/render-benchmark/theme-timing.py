"""Theme switch timing on the dedicated emulator, with the Rust renderer.

Switches the system's night mode with `cmd uimode night`, so a keyboard whose theme mode is
Follow system switches between its day and night themes, and times each switch from a marker
logged in the same device shell just before the command to the last frame the renderer posted
for it. Both ends are read from logcat's monotonic timestamps. The probe's activity does not
handle night-mode changes itself, so each switch also recreates it and the keyboard is shown
again: that is the same on every build, and is in every number. Needs ANDROID_SERIAL, the
keyboard's theme mode set to Follow system with different day and night themes, and the probe's
empty text field focused with the keyboard showing. Leaves the system's night mode off. Saves
only numbers. Run with python3 -I.
"""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import time

TAG = "aw-theme-timing"
LINE = re.compile(r"^\s*(\d+\.\d+)\s+\d+\s+\d+\s+\w\s+(\S+)\s*:\s*(\S+)")


def adb(*args):
    serial = os.environ["ANDROID_SERIAL"]
    if serial == "emulator-5554":
        raise ValueError("Reserved emulator")
    return subprocess.run(["adb", "-s", serial, *args], check=True, capture_output=True, text=True,
                          timeout=150).stdout


def events():
    """(seconds, kind) for every marker and frame, in log order."""
    log = adb("logcat", "-d", "-v", "monotonic", "-v", "usec", "-s", TAG + ":I", "aw-render:D")
    found = []
    for match in map(LINE.match, log.splitlines()):
        if match and match[2] == TAG:
            found.append((float(match[1]), "switch"))
        elif match and match[2] == "aw-render" and match[3] == "frame":
            found.append((float(match[1]), "frame"))
    return found


def switches(rows):
    """Per switch: from its marker to the last frame before the next marker, in ms."""
    found = []
    for at, kind in rows:
        if kind == "switch":
            found.append({"at": at, "last": None, "frames": 0})
        elif found:
            found[-1]["last"], found[-1]["frames"] = at, found[-1]["frames"] + 1
    return [{"ms": (s["last"] - s["at"]) * 1000 if s["last"] else None, "frames": s["frames"]} for s in found]


def percentile(values, p):
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, round(p / 100 * (len(ordered) - 1)))] if ordered else None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--switches", type=int, default=20, help="an even number, so night mode ends off")
    parser.add_argument("--pause", type=float, default=3.0)
    args = parser.parse_args()
    assert args.switches % 2 == 0, "an even number of switches"
    args.output.mkdir(parents=True, exist_ok=False)
    adb("shell", "setprop", "log.tag.aw-render", "DEBUG")
    adb("logcat", "-c")
    try:
        for n in range(args.switches):
            adb("shell", f"log -t {TAG} switch; cmd uimode night {'yes' if n % 2 == 0 else 'no'}")
            time.sleep(args.pause)
    finally:
        adb("shell", "cmd", "uimode", "night", "no")
        adb("shell", "setprop", "log.tag.aw-render", "INFO")
    found = switches(events())
    ms = [s["ms"] for s in found if s["ms"] is not None]
    result = {"switches": args.switches, "drawn": len(ms), "ms": ms, "frames": [s["frames"] for s in found],
              "p50": percentile(ms, 50), "p95": percentile(ms, 95), "max": max(ms, default=None)}
    (args.output / "theme.json").write_text(json.dumps(result, indent=2) + "\n")
    print(f"theme: {len(ms)}/{args.switches} switches drawn, p50 {result['p50']:.1f} ms, "
          f"p95 {result['p95']:.1f} ms, max {result['max']:.1f} ms", flush=True)


if __name__ == "__main__":
    main()
