"""Synthetic gesture replay. Persist editor lengths, selection offsets and SHA-256 only.

Run with python3 -I. Paths are fixed synthetic fixtures; do not record users' glide paths.
agent-device owns straight pans/taps. A multi-segment path and CANCEL use the documented
adb motionevent fallback because agent-device pan cannot express that path.
"""
import argparse
import math
import json
import os
from pathlib import Path
import subprocess
import time


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("agent")
    parser.add_argument("output")
    parser.add_argument("--glide-only", action="store_true")
    args = parser.parse_args()
    serial = os.environ["ANDROID_SERIAL"]
    if serial == "emulator-5554":
        raise SystemExit("Refusing the emulator reserved for another session")
    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)
    results = []

    def adb(*command):
        result = subprocess.run(["adb", "-s", serial, *command], capture_output=True, text=True)
        if result.returncode:
            raise RuntimeError("Targeted adb command failed: " + result.stderr)
        return result.stdout

    def agent(*command):
        result = subprocess.run([args.agent, *map(str, command)], capture_output=True, text=True)
        if result.returncode or "Error (" in result.stdout + result.stderr:
            raise RuntimeError(result.stdout + result.stderr)

    def digest(label):
        time.sleep(1)
        data = json.loads(adb("shell", "run-as", "com.jtn0123.renderprobe", "cat", "files/digest.json"))
        results.append({"step": label, **data})
        (output / "gestures.json").write_text(json.dumps(results, indent=2) + "\n")
        return data

    def path(points, cancel=False):
        # Input does not print point coordinates. No paths enter evidence logs.
        commands = [f"input motionevent DOWN {points[0][0]} {points[0][1]}"]
        for (x0, y0), (x1, y1) in zip(points, points[1:]):
            count = max(1, math.ceil(math.hypot(x1-x0, y1-y0) / 24))
            for step in range(1, count+1):
                x = round(x0 + (x1-x0)*step/count)
                y = round(y0 + (y1-y0)*step/count)
                commands.append(f"input motionevent MOVE {x} {y}")
        action = "CANCEL" if cancel else "UP"
        commands.append(f"input motionevent {action} {points[-1][0]} {points[-1][1]}")
        adb("shell", "; ".join(commands))

    # Fixed public synthetic fixtures, independent of anything entered by a person.
    fixtures = [((650, 1900), (275, 1750), (980, 1900), (925, 1750)),
                ((165, 1750), (925, 1750), (380, 1750), (980, 1900), (325, 1900)),
                ((490, 1750), (275, 1750), (215, 1900), (490, 1750))]
    for i, points in enumerate(fixtures):
        before_units = json.loads(adb("shell", "run-as", "com.jtn0123.renderprobe", "cat", "files/digest.json"))["utf16_units"]
        path(points)
        glided = digest(f"glide-{i + 1}")
        if glided["utf16_units"] - before_units < 3:
            raise RuntimeError("Glide fixture did not commit a multi-unit word")
        if i == 0:
            agent("press", 780, 1619)
            picked = digest("candidate-pick")
            if picked["sha256"] == glided["sha256"]:
                raise RuntimeError("Candidate fixture did not replace the glide word")
    if not args.glide_only:
        agent("gesture", "pan", 540, 2190, -210, 0, 300)
        digest("space-left")
        agent("gesture", "pan", 540, 2190, 140, 0, 300)
        digest("space-right")
        agent("gesture", "pan", 1015, 2060, -300, 0, 300)
        digest("delete-swipe")
        agent("gesture", "pan", 540, 2190, 280, 0, 300)
        ready = digest("backspace-repeat-ready")
        if ready["selection_start"] < 3 or ready["selection_start"] != ready["selection_end"]:
            raise RuntimeError("Backspace repeat fixture needs a nonempty prefix at the cursor")
        agent("press", 1015, 2060, "--hold-ms", 2000)
        repeated = digest("backspace-repeat")
        if ready["utf16_units"] - repeated["utf16_units"] < 2:
            raise RuntimeError("Backspace hold did not repeat deletion")
        agent("press", 70, 2060, "--count", 2, "--hold-ms", 35, "--interval-ms", 85)
        agent("press", 110, 1900)
        agent("press", 215, 1900)
        digest("double-shift")
        agent("press", 70, 2060)
        agent("gesture", "tap", 70, 2060, 110, 1900, "--hold-ms", 120)
        digest("held-shift")
        agent("gesture", "tap", 110, 1900, 215, 1900, "--hold-ms", 120)
        digest("rolling")
    before = results[-1]
    path(((650, 1900), (275, 1750), (980, 1900)), cancel=True)
    digest("cancelled-glide")
    if any(results[-1][key] != before[key] for key in ("utf16_units", "sha256", "selection_start", "selection_end")):
        raise RuntimeError("Cancelled glide changed the editor digest or selection")
    print("Recorded", len(results), "gesture digest checkpoints")


if __name__ == "__main__":
    main()
