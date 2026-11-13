"""Alternating CPU trials with the existing 300-press/eight-hold digest assertions."""
import argparse
from pathlib import Path
import runpy
import subprocess

api = runpy.run_path(str(Path(__file__).with_name("a11y_device.py")))
summary = runpy.run_path(str(Path(__file__).with_name("summarize.py")))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output")
    args = parser.parse_args()
    device = api["Device"](args.output)
    original = device.original_accessibility()
    device.save("original-accessibility.json", original)
    results = {}
    try:
        device.mute()
        for tag, level in (("aw-render", "DEBUG"), ("aw-compose", "DEBUG"),
                           ("aw-profile", "INFO"), ("aw-hybrid", "INFO"), ("aw-emoji", "INFO")):
            device.shell("setprop", "log.tag." + tag, level)
        for trial in (1, 2):
            for renderer in ("compose", "rust"):
                device.renderer(renderer)
                device.probe(no_suggestions=True)
                tree = device.capture("preflight-" + renderer + str(trial), lambda ns:
                    any(n.get("text") in ("e", "E") for n in ns))
                if any(n.get("text") == "E" for n in tree["nodes"]):
                    device.tap(70, 2060)
                if device.prefs().get("smartbar__shared_actions_expanded") == "true":
                    device.tap(1025, 1618)
                device.reset()
                device.restore_accessibility(original)
                folder = device.output / (renderer + "-" + str(trial))
                subprocess.check_call(["scripts/render-benchmark/run.sh", "adb", renderer, str(folder)])
                if renderer == "rust":
                    result = summary["native"](folder / "rust-timing.log")
                else:
                    result = summary["compose"](sorted(folder.glob("compose-gfx-*.txt")))
                    result["touch_to_commit"] = summary["compose_touch"](folder / "compose-timing.log")
                cpu = result["draw" if renderer == "rust" else "hwui_draw_recording"]
                if cpu["count"] < 100:
                    raise RuntimeError("Insufficient rendered CPU frames: " + renderer)
                results[renderer + "-" + str(trial)] = result
                device.save("timings.json", results)
                print(renderer, trial, result, flush=True)
    finally:
        device.restore_accessibility(original)


if __name__ == "__main__":
    main()
