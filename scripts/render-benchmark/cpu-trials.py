"""Alternating Compose/Rust timing trials with CPU attribution and suggestions on.

Run with python3 -I. Needs ANDROID_SERIAL (the dedicated emulator) and AGENT_DEVICE (the exact
device_open command); every UI action goes through it. Each trial lands in its own new folder
under OUTPUT; an existing folder is never reused. Saves only numeric timings and digests.
"""
import argparse
import json
import os
import re
from pathlib import Path
import runpy
import subprocess
import time

api = runpy.run_path(str(Path(__file__).with_name("a11y_device.py")))
summary = runpy.run_path(str(Path(__file__).with_name("summarize.py")))
TAGS = ("aw-render", "aw-compose", "aw-profile", "aw-hybrid", "aw-emoji")


def tags(device, profile):
    levels = {"aw-render": "DEBUG", "aw-compose": "DEBUG",
              "aw-profile": "DEBUG" if profile else "INFO"}
    for tag in TAGS:
        device.shell("setprop", "log.tag." + tag, levels.get(tag, "INFO"))


def prepare(device, renderer, original, trial):
    device.renderer(renderer)
    device.probe(no_suggestions=False)
    tree = device.capture("preflight-" + renderer + str(trial),
                          lambda ns: any(n.get("text") in ("e", "E") for n in ns))
    if any(n.get("text") == "E" for n in tree["nodes"]):
        device.tap(70, 2060)
    if device.prefs().get("smartbar__shared_actions_expanded") == "true":
        device.tap(1025, 1618)
    # Warm the shaping and glyph paths, clear, then let the strip settle.
    for x in (60, 165, 275):
        device.tap(x, 1750)
    device.reset()
    device.restore_accessibility(original)
    time.sleep(1.5)


def record(agent, renderer, folder):
    """Run one trial. Rust must type the expected text; frozen Compose is a timing reference
    whose committed text differs on current main, so its digests are recorded, not enforced."""
    code = subprocess.call(["scripts/render-benchmark/run.sh", agent, renderer, str(folder)])
    timing = folder / (renderer + "-timing.log")
    if code and (renderer == "rust" or not timing.exists()
                 or re.search(r"FATAL|AndroidRuntime|draw_error|post_error", timing.read_text())):
        raise RuntimeError("Trial failed: " + str(folder))
    digests = {name: json.loads((folder / (renderer + "-" + name + "-digest.json")).read_text())
               for name in ("typing", "popup-final")}
    return {name: {key: value[key] for key in ("utf16_units", "sha256")}
            for name, value in digests.items()} | {"expected_text": code == 0}


def summarize(folder, renderer):
    if renderer == "rust":
        result = summary["native"](folder / "rust-timing.log")
        result["profile"] = summary["profile"](folder / "rust-timing.log")
        cpu = result["draw"]
    else:
        result = summary["compose"](sorted(folder.glob("compose-gfx-*.txt")))
        result["touch_to_commit"] = summary["compose_touch"](folder / "compose-timing.log")
        cpu = result["hwui_draw_recording"]
    if cpu["count"] < 100:
        raise RuntimeError("Insufficient rendered CPU frames: " + renderer)
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output")
    parser.add_argument("--trials", type=int, default=2)
    parser.add_argument("--no-profile", action="store_true")
    parser.add_argument("--rust-only", action="store_true",
                        help="iterate on Rust alone; the evidence alternates both renderers")
    args = parser.parse_args()
    agent = os.environ["AGENT_DEVICE"]
    device = api["Device"](args.output)
    original = device.original_accessibility()
    device.save("original-accessibility.json", original)
    results = {}
    try:
        device.mute()
        tags(device, not args.no_profile)
        for trial in range(1, args.trials + 1):
            for renderer in ("rust",) if args.rust_only else ("compose", "rust"):
                folder = device.output / (renderer + "-" + str(trial))
                if folder.exists():
                    raise RuntimeError("Refusing to overwrite " + str(folder))
                prepare(device, renderer, original, trial)
                digests = record(agent, renderer, folder)
                results[renderer + "-" + str(trial)] = summarize(folder, renderer) | {"digests": digests}
                device.save("timings.json", results)
                print(renderer, trial, "done", flush=True)
    finally:
        device.restore_accessibility(original)
        for tag in TAGS:
            device.shell("setprop", "log.tag." + tag, "INFO")


if __name__ == "__main__":
    main()
