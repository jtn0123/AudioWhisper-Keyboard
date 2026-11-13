"""Capture empty-editor mode/theme pairs through the device_open agent-device command."""
import argparse
from pathlib import Path
import runpy
import subprocess
import time

modes = runpy.run_path(str(Path(__file__).with_name("modes.py")))
api = modes["api"]
Device, SERVICE = api["Device"], api["SERVICE"]


def theme(device, name):
    device.services(SERVICE)
    device.shell("am", "start", "-W", "-a", "android.intent.action.VIEW", "-c", "android.intent.category.BROWSABLE",
                 "-d", "ui://florisboard/settings/theme", "-p", api["PKG"])
    wanted = '"ALWAYS_' + name.upper() + '"'
    if device.prefs().get("theme__mode") != wanted:
        for label in ("Theme mode", "Always " + name, "OK"):
            if device.prefs().get("theme__mode") == wanted:
                break
            def click():
                request = device.request(setting=label)
                return device.wait(lambda: device.response(request), "Setting response absent").get("ok")
            device.wait(click, "Theme setting absent: " + label)
        device.wait(lambda: device.prefs().get("theme__mode") == wanted, "Theme mode was not persisted")
    expected = '"org.florisboard.themes:floris_' + name + '"'
    # Unset ids use AppPrefs' bundled Floris Day/Night defaults.
    if device.prefs().get("theme__" + name + "_theme_id", expected) != expected:
        raise RuntimeError("Select bundled Floris " + name + " before visual replay")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("agent")
    parser.add_argument("output")
    args = parser.parse_args()
    device = Device(args.output)
    original = device.original_accessibility()
    capture_prefs = device.capture_preferences()
    try:
        # Establish the screenshot target before preparing the editor. Reopening it after
        # probe() launches a second activity and destroys the already-prepared IME surface.
        subprocess.check_output([args.agent, "open", api["PROBE"], "--no-test-ime"], text=True)
        device.shell("setprop", "log.tag.aw-profile", "DEBUG")
        device.shell("setprop", "log.tag.aw-render", "DEBUG")
        for name in ("day", "night"):
            for renderer in ("compose", "rust"):
                device.renderer(renderer)
                theme(device, name)
                device.probe(no_suggestions=True)
                device.services(SERVICE)
                for mode in ("symbols", "symbols2", "numeric", "phone", "password"):
                    device.adb("logcat", "-c")
                    modes["prepare"](device, renderer, mode)
                    device.restore_accessibility(original)
                    if renderer == "rust":
                        device.wait(lambda: "frame draw_ns=" in device.adb("logcat", "-d", "-s", "aw-render:D"),
                                    "Native surface did not post a frame: " + mode)
                    time.sleep(.5)
                    path = device.output.resolve() / (renderer + "-" + name + "-" + mode + ".png")
                    result = subprocess.check_output([args.agent, "screenshot", str(path)], text=True)
                    (path.with_suffix(".capture.json")).write_text(result)
                    logs = device.adb("logcat", "-d", "-s", "aw-render:D", "aw-profile:D")
                    (path.with_suffix(".native.log")).write_text(logs)
                    if any(error in logs for error in ("draw_error", "post_error")):
                        raise RuntimeError("Native draw/post failure: " + mode)
                    device.services(SERVICE)
        # Leave this session idle for the following adb-only runs. Closing the probe
        # app would also stop its dump service; device_close releases the session at cleanup.
        for name in ("day", "night"):
            for mode in ("symbols", "symbols2", "numeric", "phone", "password"):
                paths = [str(device.output / (r + "-" + name + "-" + mode + ".png")) for r in ("compose", "rust")]
                pair = str(device.output / (name + "-" + mode + "-pair.png"))
                subprocess.check_call(["swift", "scripts/render-benchmark/pair.swift", *paths, pair])
    finally:
        theme(device, "day")
        device.restore_capture_preferences(capture_prefs)
        device.restore_accessibility(original)


if __name__ == "__main__":
    main()
