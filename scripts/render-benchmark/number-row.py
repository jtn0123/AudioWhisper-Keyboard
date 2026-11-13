"""Number-row mode-switch control; uses empty-editor trees to locate public fixture keys."""
import hashlib
from pathlib import Path
import runpy

api = runpy.run_path(str(Path(__file__).with_name("a11y_device.py")))
Device, SERVICE = api["Device"], api["SERVICE"]


def target(tree, label):
    nodes = tree["nodes"]
    matches = [n for n in nodes if n.get("text") == label and n["bounds"][1] > 1300]
    node = min(matches, key=lambda n: n["bounds"][1])
    b = node["bounds"]
    return ((b[0] + b[2]) // 2, (b[1] + b[3]) // 2)


def number_row(device, enabled):
    device.services(SERVICE)
    device.shell("am", "start", "-W", "-a", "android.intent.action.VIEW", "-c", "android.intent.category.BROWSABLE",
                 "-d", "ui://florisboard/settings/keyboard", "-p", api["PKG"])
    if (device.prefs().get("keyboard__number_row") == "true") != enabled:
        def toggle():
            request = device.request(setting="Number row")
            return device.wait(lambda: device.response(request), "Number-row setting not found").get("ok")
        device.wait(toggle, "Number-row toggle failed")
        device.wait(lambda: (device.prefs().get("keyboard__number_row") == "true") == enabled,
                    "Number row was not persisted")


def replay(device, renderer):
    device.renderer(renderer)
    number_row(device, True)
    device.probe(no_suggestions=True)
    tree = device.capture(renderer + "-number-row-initial", lambda ns: any(n.get("text") in ("e", "E") for n in ns))
    if any(n.get("text") == "E" for n in tree["nodes"]):
        device.tap(70, 2060)
    tree = device.capture(renderer + "-number-row", lambda ns: any(n.get("text") == "e" for n in ns))
    one, letter = target(tree, "1"), target(tree, "e")
    symbols = (70, 2190)
    device.tap(*symbols)
    first = device.capture(renderer + "-number-row-symbols")
    first_one = target(first, "1")
    second_switch = target(first, "More symbols" if renderer == "rust" else "=\\<")
    device.tap(*second_switch)
    second = device.capture(renderer + "-number-row-symbols2")
    tilde = target(second, "Tilde" if renderer == "rust" else "~")
    device.tap(*symbols)
    device.restore_accessibility({"enabled_accessibility_services": "null", "accessibility_enabled": "0"})
    for position in (letter, one, symbols, first_one, second_switch, tilde, symbols, letter):
        device.hold(*position, 35)
    expected = hashlib.sha256(b"e11~e").hexdigest()
    device.wait(lambda: device.digest()["sha256"] == expected, "Number-row fixture mismatch")
    device.save(renderer + "-digest.json", device.digest())
    print(renderer, device.digest()["sha256"], flush=True)


def main():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("output")
    args = parser.parse_args()
    device = Device(args.output)
    original = device.original_accessibility()
    original_row = device.prefs().get("keyboard__number_row") == "true"
    capture_prefs = device.capture_preferences()
    try:
        for renderer in ("compose", "rust"):
            replay(device, renderer)
    finally:
        number_row(device, original_row)
        device.restore_capture_preferences(capture_prefs)
        device.restore_accessibility(original)


if __name__ == "__main__":
    main()
