"""Settings-only TalkBack replay. Persist only digests, cursor offsets and fixed state words."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import runpy
import subprocess
import time

api = runpy.run_path(str(Path(__file__).with_name("a11y_device.py")))
Device, SERVICE = api["Device"], api["SERVICE"]
TALKBACK = "com.google.android.marvin.talkback"
STATE_WORDS = ("Shift enabled", "Shift disabled", "Caps lock enabled")


def component(device):
    package = device.shell("dumpsys", "package", TALKBACK)
    # Equivalent to dumpsys package | grep -i TalkBackService, retaining the exact component.
    matches = set(re.findall(r"com\.google\.android\.marvin\.talkback/[\w.$]*TalkBackService", package))
    if len(matches) != 1:
        raise RuntimeError("TalkBackService component is absent or ambiguous")
    return matches.pop()


def run(device, renderer, service, duration):
    device.services(SERVICE)
    device.renderer(renderer)
    device.probe()
    # Prepare the same collapsed toolbar and lower-case keyboard before enabling TalkBack.
    if device.prefs().get("smartbar__shared_actions_expanded") == "true":
        device.tap(1025, 1618)
    device.services(service)
    device.wait(lambda: "touchExplorationEnabled=true" in device.shell("dumpsys", "accessibility"),
                "TalkBack did not enable touch exploration")
    # The notification permission was handled with pm before activation. Keep the
    # already focused editor and require it to remain unobscured.
    device.wait(lambda: api["PROBE"] in device.shell("dumpsys", "activity", "activities")
                .split("topResumedActivity=")[-1].split("\n")[0], "Probe is obscured by TalkBack setup")
    states = device.shell("dumpsys", "accessibility")
    (device.output / (renderer + "-talkback-enabled.txt")).write_text(states)
    results = []

    def checkpoint(name, expected=None):
        # Editor callbacks publish after the touch command; require the digest to settle.
        if api["PROBE"] not in device.shell("dumpsys", "activity", "activities").split("topResumedActivity=")[-1].split("\n")[0]:
            raise RuntimeError("TalkBack setup obscured the editor at " + name)
        previous = None
        stable_since = time.monotonic()
        deadline = stable_since + 4
        while time.monotonic() < deadline:
            current = device.digest()
            if current != previous:
                previous = current
                stable_since = time.monotonic()
            if time.monotonic() - stable_since >= .4:
                break
            time.sleep(.05)
        row = {"step": name, **current}
        if expected is not None:
            row["expected_units"] = len(expected)
            row["expected_sha256"] = hashlib.sha256(expected.encode()).hexdigest()
            row["matches_expected"] = (current["utf16_units"] == len(expected)
                and current["sha256"] == row["expected_sha256"]
                and current["selection_start"] == current["selection_end"] == len(expected))
        results.append(row)
        device.save(renderer + "-talkback.json", results)
        print(renderer, name, current["utf16_units"], current["sha256"], flush=True)

    checkpoint("initial", "")
    for name, x, y, expected in (("key-1", 435, 2060, "c"), ("key-2", 110, 1900, "ca"),
                                 ("word", 490, 1750, "cat")):
        device.hold(x, y, duration)
        checkpoint(name, expected)
    device.hold(70, 2060, duration)
    checkpoint("shift")
    device.hold(110, 1900, duration)
    checkpoint("capital", "catA")
    # First focus e, then double-tap with the second tap held and dragged to an accent.
    device.hold(275, 1750, duration)
    checkpoint("accent-focus")
    device.shell("sh", "-c", "input tap 275 1750; sleep 0.08; input motionevent DOWN 275 1750; sleep 0.8; input motionevent MOVE 165 1560; input motionevent UP 165 1560")
    checkpoint("accent-attempt")
    # Explore/lift the expansion control, then Undo at its configured visible slot.
    device.hold(1025, 1618, duration)
    checkpoint("expand")
    device.hold(475, 1618, duration)
    checkpoint("undo")
    # A double-tap-and-hold on Shift is the accessible caps-lock action.
    device.hold(70, 2060, duration)
    device.shell("sh", "-c", "input tap 70 2060; sleep 0.08; input swipe 70 2060 70 2060 900")
    checkpoint("caps-lock-attempt")
    # Do not save general TalkBack logs: they may contain the editor's character echo.
    logs = device.adb("logcat", "-d", "-v", "brief")
    announcements = []
    for line in logs.splitlines():
        if any(tag in line for tag in ("TalkBack", "SpeechController", "Feedback")):
            announcements += [word for word in STATE_WORDS if word in line]
    device.save(renderer + "-announcements.json", {
        "fixed_state_words_in_talkback_log": sorted(set(announcements)),
        "shift_confirmed": "Shift enabled" in announcements,
        "caps_confirmed": "Caps lock enabled" in announcements,
        "boundary": "Bound service/touch exploration proves activation, not speech output. No general log text retained.",
    })
    return results


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output")
    parser.add_argument("--hold-ms", type=int, default=400)
    parser.add_argument("--renderer", choices=("compose", "rust"))
    args = parser.parse_args()
    device = Device(args.output)
    original = device.original_accessibility()
    device.save("original-accessibility.json", original)
    try:
        device.mute()
        package = device.shell("dumpsys", "package", TALKBACK)
        notifications_granted = "android.permission.POST_NOTIFICATIONS: granted=true" in package
        if not notifications_granted:
            device.shell("pm", "grant", TALKBACK, "android.permission.POST_NOTIFICATIONS")
        service = component(device)
        device.save("talkback-component.json", {"component": service})
        results = {}
        for renderer in ([args.renderer] if args.renderer else ["compose", "rust"]):
            results[renderer] = run(device, renderer, service, args.hold_ms)
        comparisons = []
        for a, b in zip(results.get("compose", []), results.get("rust", [])):
            fields = ("utf16_units", "sha256", "selection_start", "selection_end")
            comparisons.append({"step": a["step"], "matches": all(a[k] == b[k] for k in fields)})
        device.save("talkback-comparison.json", comparisons)
    finally:
        device.restore_accessibility(original)
        if not notifications_granted:
            device.shell("pm", "revoke", TALKBACK, "android.permission.POST_NOTIFICATIONS")
        device.save("restored-accessibility.json", device.original_accessibility())
        (device.output / "accessibility-after.txt").write_text(device.shell("dumpsys", "accessibility"))


if __name__ == "__main__":
    main()
