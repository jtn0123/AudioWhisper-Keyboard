"""Where a mode or subtype switch's time goes, on the dedicated emulator, with the Rust renderer.

Taps as rebuild-timing.py does (the same keys, hold and tap grouping) and reports two things per
kind of switch:

* cpu: the keyboard process's CPU time per switch, all threads and the main thread, from
  /proc through run-as, with the same read over 15 idle seconds first. It does not depend on how
  late the emulator delivers a tap. simpleperf's default events cannot count it: the emulator's
  kernel cannot count user space alone, and SELinux denies the shell the kernel's events. Its
  software clock can, without root, to show where the time goes: `simpleperf record --app
  com.jtn0123.audiowhisper.keyboard -e cpu-clock -f 1000 --call-graph dwarf`.
* split: per tap, from the renderer's frame log, input (the release's touch to its frame's
  choreographer callback, which includes work done before that frame), build (that frame's post
  to the rebuilt frame's callback) and draw (the rebuilt frame's callback to its post); medians.

Needs ANDROID_SERIAL, AGENT_DEVICE, the probe's empty field focused and two subtypes with the
language key showing. Saves only numbers. Run with python3 -I.
"""
import argparse
import json
from pathlib import Path
import re
import runpy
import statistics
import time

timing = runpy.run_path(str(Path(__file__).with_name("rebuild-timing.py")))
adb, press, KEYS, TAP_NS = timing["adb"], timing["press"], timing["KEYS"], timing["TAP_NS"]
PKG = "com.jtn0123.audiowhisper.keyboard"
HZ = 100
FRAME = re.compile(r"frame draw_ns=(\d+) post_ns=(\d+) touch_ns=(\d+) choreographer_ns=(\d+)")


def ticks():
    """User and system clock ticks of the keyboard process: all threads, and the main thread."""
    pid = adb("shell", "pidof", PKG).split()[0]
    rows = adb("shell", f"run-as {PKG} cat /proc/{pid}/stat /proc/{pid}/task/{pid}/stat").splitlines()
    fields = [row.rsplit(")", 1)[1].split() for row in rows]
    return [int(f[11]) + int(f[12]) for f in fields]


def frames():
    lines = adb("logcat", "-d", "-s", "aw-render:D").splitlines()
    keys = ("post", "touch", "chor")
    return [dict(zip(keys, map(int, m.groups()[1:]))) for m in map(FRAME.search, lines) if m]


def split(rows):
    """Per tap: its last touch's frame and the rebuilt keyboard's frame, which carries no touch."""
    taps = []
    for row in rows:
        if not row["touch"]:
            if taps:
                taps[-1]["rebuilt"] = row
            continue
        if not taps or abs(row["touch"] - taps[-1]["first"]) >= TAP_NS:
            taps.append({"first": row["touch"], "release": row, "rebuilt": None})
        elif row["touch"] >= taps[-1]["release"]["touch"]:
            taps[-1]["release"] = row
    return [{"input": (t["release"]["chor"] - t["release"]["touch"]) / 1e6,
             "build": (t["rebuilt"]["chor"] - t["release"]["post"]) / 1e6,
             "draw": (t["rebuilt"]["post"] - t["rebuilt"]["chor"]) / 1e6}
            for t in taps if t["rebuilt"]]


def phase(name, count):
    adb("logcat", "-c")
    before = ticks()
    for _ in range(count):
        press(*KEYS[name])
        time.sleep(1.5)
    time.sleep(2)
    after = ticks()
    taps = split(frames())
    return {"switches": count, "taps": len(taps),
            "cpu_ms_per_switch": {"all": (after[0] - before[0]) * 1000 / HZ / count,
                                  "main": (after[1] - before[1]) * 1000 / HZ / count},
            "split_median_ms": {k: statistics.median(t[k] for t in taps) for k in ("input", "build", "draw")}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--switches", type=int, default=20, help="an even number, so both end where they began")
    args = parser.parse_args()
    assert args.switches % 2 == 0, "an even number of switches"
    args.output.mkdir(parents=True, exist_ok=False)
    adb("shell", "setprop", "log.tag.aw-render", "DEBUG")
    try:
        before = ticks()
        time.sleep(15)
        idle = [(b - a) * 1000 / HZ for a, b in zip(before, ticks())]
        result = {"idle_15s_cpu_ms": {"all": idle[0], "main": idle[1]}}
        for name in ("mode", "subtype"):
            result[name] = phase(name, args.switches)
            print(name, json.dumps(result[name]), flush=True)
    finally:
        adb("shell", "setprop", "log.tag.aw-render", "INFO")
    (args.output / "cost.json").write_text(json.dumps(result, indent=1) + "\n")


if __name__ == "__main__":
    main()
