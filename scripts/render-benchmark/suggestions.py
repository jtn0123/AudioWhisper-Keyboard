"""Public autocorrection/candidate fixtures. Persist digests only, and restore device state."""
import hashlib
from pathlib import Path
import runpy
import sys
import time
import uuid

chrome = runpy.run_path(str(Path(__file__).with_name("chrome.py")))
api = runpy.run_path(str(Path(__file__).with_name("a11y_device.py")))


def replay(device, renderer):
    device.renderer(renderer)
    device.probe()
    device.services(api["SERVICE"])
    results = {}
    keys = {"t": (485, 1750), "e": (275, 1750), "h": (650, 1900),
            "l": (975, 1900), "o": (915, 1750)}
    for label, fixture in (("autocorrect", "teh"), ("candidate", "helo")):
        device.reset()
        tree = device.wait(lambda: (t if (t := device.tree()).get("nodes") else None), "Empty keyboard unavailable")
        if any(n.get("text") == "E" for n in tree["nodes"]):
            device.tap(70, 2060)
        # The observer is not asked for typed-state trees. Fixed strip coordinates
        # are checked by the resulting commit digest.
        for char in fixture:
            device.tap(*keys[char])
            time.sleep(.12)
        device.wait(lambda: device.digest()["utf16_units"] == len(fixture), "Fixture typing incomplete")
        time.sleep(.8)
        if label == "autocorrect":
            device.tap(540, 2200)
            expected = hashlib.sha256(b"the ").hexdigest()
        else:
            request = uuid.uuid4().hex
            device.shell("am", "broadcast", "-a", api["PROBE"] + ".IME", "-p", api["PROBE"],
                         "--es", "request", request, "--ez", "nlpStrip", "true")
            strip = device.wait(lambda: device.response(request), "Candidate strip probe absent")
            device.save(renderer + "-candidate-strip.json", strip)
            device.tap(365, 1619)
            expected = hashlib.sha256(b"help").hexdigest()
        try:
            device.wait(lambda: device.digest()["sha256"] == expected, "Suggestion commit digest mismatch: " + label)
        finally:
            device.save(renderer + "-" + label + "-digest.json", device.digest())
        results[label] = device.digest()
    return results


def main():
    device = api["Device"](Path(sys.argv[1]))
    access = device.original_accessibility()
    expanded = device.prefs().get("smartbar__shared_actions_expanded", "false") == "true"
    prototype = device.prefs().get("keyboard__rust_renderer_prototype") == "true"
    try:
        device.mute()
        chrome["set_preferences"](device, {"smartbar__shared_actions_expanded": False})
        results = {renderer: replay(device, renderer) for renderer in ("compose", "rust")}
        assert results["compose"] == results["rust"], "Renderer suggestion fixtures differ"
        device.save("suggestions.json", results)
        print("Both renderers matched autocorrect and candidate-pick digests")
    finally:
        chrome["set_preferences"](device, {"smartbar__shared_actions_expanded": expanded})
        device.renderer("rust" if prototype else "compose")
        device.restore_accessibility(access)


if __name__ == "__main__":
    main()
