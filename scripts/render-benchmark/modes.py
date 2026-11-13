"""Fixed public mode fixtures. Persist only editor digests/offsets, never editor contents."""
import argparse
import hashlib
import json
from pathlib import Path
import runpy
import re
import time

api = runpy.run_path(str(Path(__file__).with_name("a11y_device.py")))
Device, SERVICE = api["Device"], api["SERVICE"]
# Coordinates are for the documented 1080 x 2400 aw_keyboard AVD. Holds are short taps.
SYMBOLS, SECOND, ABC, ADVANCED, PHONE = (70, 2190), (70, 2060), (70, 2190), (335, 2190), (139, 2190)
FIXTURES = {
    "characters": ("text", [], [(435, 2060), (110, 1900), (490, 1750)], "cat"),
    "symbols": ("text", [SYMBOLS], [(60, 1750), (165, 1750), (60, 1900), (165, 1900),
        SECOND, (60, 1750), SECOND, (275, 1750), ABC, (275, 1750)], "12@#~3e"),
    "symbols2": ("text", [SYMBOLS, SECOND], [(60, 1750), (165, 1750), (275, 1750),
        SECOND, (380, 1750), SECOND, (595, 1750), ABC, (275, 1750)], "~`|4πe"),
    "numeric": ("number", [], [(139, 1750), (406, 1750), (674, 1750),
        (139, 1900), (406, 1900), (406, 2190)], "123450"),
    "numeric_advanced": ("text", [SYMBOLS, ADVANCED], [(295, 1750), (540, 1750), (89, 1750),
        (784, 1750), (349, 2190), (380, 1750), ABC, (275, 1750)], "12+34e"),
    "phone": ("phone", [], [(139, 1750), (406, 1750), (674, 1750), PHONE,
        (139, 2060), (674, 2060), PHONE, (406, 2190)], "123*#0"),
    "phone2": ("phone", [PHONE], [(139, 1750), (674, 1750), (139, 2060), (674, 2060),
        PHONE, (139, 1750), PHONE, (406, 2190)], "()*#1+"),
    "password": ("password", [], [(435, 2060), (110, 1900), SYMBOLS, (60, 1750),
        (165, 1900), ABC, (490, 1750)], "ca1#t"),
}


def prepare(device, renderer, mode):
    field, switches, _, _ = FIXTURES[mode]
    device.reset(no_suggestions=True, field=field)
    time.sleep(.5)
    if field in ("text", "password"):
        tree = device.capture(renderer + "-" + mode + "-start", lambda ns:
            any(n.get("text") in ("e", "E") for n in ns))
        if any(n.get("text") == "E" for n in tree["nodes"]):
            device.tap(70, 2060)
    if device.prefs().get("smartbar__shared_actions_expanded") == "true":
        device.tap(1025, 1618)
    for x, y in switches:
        device.hold(x, y, 35)
    tree = device.capture(renderer + "-" + mode)
    virtual = sum(n.get("textEntryKey", False) for n in tree["nodes"])
    if renderer == "rust" and virtual == 0:
        raise RuntimeError("Rust mode silently fell back to Compose: " + mode)
    return virtual


def replay(device, renderer):
    device.renderer(renderer)
    device.probe(no_suggestions=True)
    device.capture(renderer + "-surface-ready")
    results = {}
    for mode, (_, _, taps, expected) in FIXTURES.items():
        device.adb("logcat", "-c")
        virtual = prepare(device, renderer, mode)
        original = device.original_accessibility()
        device.restore_accessibility({"enabled_accessibility_services": "null", "accessibility_enabled": "0"})
        try:
            for x, y in taps:
                device.hold(x, y, 35)
            expected_hash = hashlib.sha256(expected.encode()).hexdigest()
            device.wait(lambda: device.digest()["sha256"] == expected_hash, "Mode digest mismatch: " + mode)
            digest = device.digest()
            if digest["selection_start"] != len(expected) or digest["selection_end"] != len(expected):
                raise RuntimeError("Cursor mismatch: " + mode)
            logs = device.adb("logcat", "-d", "-s", "aw-profile:D", "aw-render:D")
            lifecycle = [line for line in logs.splitlines() if re.search(r"view event=|window (acquired|released)", line)]
            device.save(renderer + "-" + mode + "-lifecycle.json", lifecycle)
            if renderer == "rust" and any(re.search(r"view event=[234567] |window (acquired|released)", line) for line in lifecycle):
                raise RuntimeError("Surface recreated during mode switches: " + mode)
            results[mode] = {**digest, "virtual_keys": virtual, "matches_fixture": True,
                             "surface_recreated_during_replay": False if renderer == "rust" else None}
            device.save(renderer + "-modes.json", results)
            print(renderer, mode, digest["sha256"], flush=True)
        finally:
            device.restore_accessibility(original)
    return results


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
        device.shell("setprop", "log.tag.aw-profile", "DEBUG")
        results = {r: replay(device, r) for r in ([args.renderer] if args.renderer else ("compose", "rust"))}
        fields = ("utf16_units", "sha256", "selection_start", "selection_end")
        comparison = {mode: all(results["compose"][mode][k] == results["rust"][mode][k] for k in fields)
                      for mode in FIXTURES} if len(results) == 2 else {}
        device.save("comparison.json", comparison)
        if not all(comparison.values()):
            raise RuntimeError("Per-mode parity failed")
    finally:
        device.restore_capture_preferences(capture_prefs)
        device.restore_accessibility(original)


if __name__ == "__main__":
    main()
