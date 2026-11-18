"""Serial-scoped harness operations. Never read or persist editor contents."""
import json
import os
from pathlib import Path
import shlex
import subprocess
import time
import uuid
import runpy

agent_ui = runpy.run_path(str(Path(__file__).with_name('agent-ui.py')))

PKG = "com.jtn0123.audiowhisper.keyboard"
PROBE = "com.jtn0123.renderprobe"
SERVICE = PROBE + "/.ProbeAccessibilityService"
PREFS = "jetpref_datastore/florisboard-app-prefs.jetpref"


class Device:
    def __init__(self, output):
        self.serial = os.environ["ANDROID_SERIAL"]
        if self.serial == "emulator-5554":
            raise ValueError("Reserved emulator")
        self.output = Path(output)
        self.output.mkdir(parents=True, exist_ok=True)

    def adb(self, *args):
        return subprocess.check_output(["adb", "-s", self.serial, *map(str, args)], text=True)

    def shell(self, *args):
        if os.environ.get('AGENT_DEVICE'):
            if args[:2] == ('am', 'start'):
                return agent_ui['launch'](args[2:])
            if args[0] == 'input' and args[1] != 'motionevent':
                return agent_ui['touch'](args[1:])
        return self.adb("shell", shlex.join(list(map(str, args))))

    def save(self, name, value):
        (self.output / name).write_text(json.dumps(value, indent=2) + "\n")

    def wait(self, check, message, timeout=10):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            result = check()
            if result:
                return result
            time.sleep(.05)
        raise RuntimeError(message)

    def original_accessibility(self):
        return {key: self.shell("settings", "get", "secure", key).strip() for key in
                ("enabled_accessibility_services", "accessibility_enabled")}

    def restore_accessibility(self, original):
        for key, value in original.items():
            if value == "null":
                self.shell("settings", "delete", "secure", key)
            else:
                self.shell("settings", "put", "secure", key, value)
        self.wait(lambda: self.original_accessibility() == original, "Settings restoration failed")

    def services(self, component):
        self.shell("settings", "put", "secure", "enabled_accessibility_services", component)
        self.shell("settings", "put", "secure", "accessibility_enabled", 1)
        label = "Renderer probe" if component == SERVICE else "TalkBack"
        self.wait(lambda: any("Bound services:" in line and "label=" + label in line
                             for line in self.shell("dumpsys", "accessibility").splitlines()),
                  "Accessibility service did not bind")

    def mute(self):
        self.shell("cmd", "notification", "set_dnd", "off")
        # `cmd media_session volume --set` reports success but changes nothing on the android-36
        # AVD, so set through AudioService and fail unless the readback matches. Accessibility
        # (10) and alarm (4) have a minimum of 1.
        results = {}
        for stream, target in ((1, 0), (2, 0), (3, 0), (5, 0), (10, 1), (4, 1)):
            response = self.shell("cmd", "audio", "set-volume", stream, target)
            level = self.shell("cmd", "audio", "get-stream-volume", stream).rsplit("->", 1)[-1].strip()
            if level != str(target):
                raise RuntimeError(f"Stream {stream} is at {level or 'unknown'}, not {target}")
            results[str(stream)] = response + f"level {level}\n"
        self.save("muting.json", results)

    def prefs(self):
        return {parts[1]: parts[2] for row in self.adb("exec-out", "run-as", PKG, "cat", PREFS).splitlines()
                if len(parts := row.split(";", 2)) == 3}

    def toggle_preference(self, route, label, key, enabled, default=True):
        self.services(SERVICE)
        self.shell("am", "start", "-W", "-a", "android.intent.action.VIEW", "-c", "android.intent.category.BROWSABLE",
                   "-d", "ui://florisboard/settings/" + route, "-p", PKG)
        if (self.prefs().get(key, str(default).lower()) == "true") == enabled:
            return
        def toggle():
            request = self.request(setting=label)
            return self.wait(lambda: self.response(request), "Preference response absent").get("ok")
        self.wait(toggle, "Preference not found: " + label)
        self.wait(lambda: (self.prefs().get(key, str(default).lower()) == "true") == enabled, "Preference did not persist")

    def capture_preferences(self):
        # TYPE_TEXT_FLAG_NO_SUGGESTIONS only applies to text fields. Numeric/phone
        # fields otherwise retain predictions from a previous public fixture.
        specs = (("typing", "Display suggestions", "suggestion__enabled"),
                 ("media", "Enable emoji suggestions", "emoji__suggestion_enabled"))
        original = [(route, label, key, self.prefs().get(key, "true") == "true") for route, label, key in specs]
        self.save("original-capture-preferences.json", original)
        access = self.original_accessibility()
        try:
            for route, label, key, _ in original:
                self.toggle_preference(route, label, key, False)
        except Exception:
            self.restore_capture_preferences(original)
            self.restore_accessibility(access)
            raise
        return original

    def restore_capture_preferences(self, original):
        for route, label, key, enabled in original:
            self.toggle_preference(route, label, key, enabled)

    def request(self, setting=None, delay=0):
        request = uuid.uuid4().hex
        args = ["am", "broadcast", "-a", PROBE + ".IME", "-p", PROBE,
                "--es", "request", request, "--ei", "delay", delay]
        if setting:
            args += ["--es", "setting", setting]
            if os.environ.get('AGENT_DEVICE'):
                args += ['--ez', 'geometryOnly', 'true']
        self.shell(*args)
        if setting and os.environ.get('AGENT_DEVICE'):
            reply = self.wait(lambda: self.response(request), 'Setting geometry absent')
            if reply.get('found'):
                x1, y1, x2, y2 = reply['bounds']
                self.tap((x1 + x2) // 2, (y1 + y2) // 2)
        return request

    def response(self, request):
        result = subprocess.run(["adb", "-s", self.serial, "exec-out", "run-as", PROBE,
                                 "cat", "files/" + request + ".json"], capture_output=True, text=True)
        if result.returncode:
            return None
        try:
            data = json.loads(result.stdout)
        except json.JSONDecodeError:
            return None
        if data["request"] != request:
            raise RuntimeError("Stale probe response")
        return data

    def tree(self):
        request = self.request()
        return self.wait(lambda: self.response(request), "Dump service did not respond")

    def capture(self, name, predicate=lambda nodes: True):
        previous = None
        stable_since = time.monotonic()

        def ready():
            nonlocal previous, stable_since
            data = self.tree()
            nodes = data.get("nodes")
            valid = nodes and all(n["bounds"][3] <= 2400 for n in nodes if n.get("visible")) and predicate(nodes)
            if nodes != previous or not valid:
                previous = nodes
                stable_since = time.monotonic()
            return data if valid and time.monotonic() - stable_since >= .4 else None
        data = self.wait(ready, "Expected stable IME state not reached: " + name)
        self.save(name + ".json", data)
        print(name, len(data["nodes"]), flush=True)
        return data

    def renderer(self, renderer):
        enabled = renderer == "rust"
        self.services(SERVICE)
        self.shell("am", "start", "-W", "-a", "android.intent.action.VIEW", "-c", "android.intent.category.BROWSABLE", "-d",
                   "ui://florisboard/settings/keyboard", "-p", PKG)
        if (self.prefs().get("keyboard__rust_renderer_prototype", "true") == "true") == enabled:
            return

        def toggle():
            request = self.request(setting="Draw the keyboard in Rust")
            result = self.wait(lambda: self.response(request), "Settings request timed out")
            return result.get("ok")
        try:
            self.wait(toggle, "Renderer switch not found")
        except RuntimeError as failure:
            if str(failure) != "Renderer switch not found":
                raise
            # A warm settings deep link can have an empty navigation host. Select the same
            # persisted fixture flag on a fresh process; digest/virtual-key checks still apply.
            raw = self.adb("exec-out", "run-as", PKG, "cat", PREFS)
            key = "keyboard__rust_renderer_prototype"
            rows = [row for row in raw.splitlines() if len(parts := row.split(";", 2)) != 3 or parts[1] != key]
            rows.append("b;" + key + ";" + str(enabled).lower())
            self.shell("am", "force-stop", PKG)
            subprocess.run(["adb", "-s", self.serial, "shell", "run-as " + PKG + " sh -c " + shlex.quote("cat > " + PREFS)],
                           input=("\n".join(rows) + "\n").encode(), check=True)
            self.shell("ime", "set", PKG + "/dev.patrickgold.florisboard.FlorisImeService")
            self.save("renderer-selection-" + renderer + ".json", {"renderer": renderer, "via": "persisted-fixture-flag", "warm_settings_control_missing": True})
        self.wait(lambda: (self.prefs().get("keyboard__rust_renderer_prototype", "true") == "true") == enabled,
                  "Renderer setting was not persisted")

    def probe(self, no_suggestions=False, field="text", extras=()):
        # Unbind our observer before replacing its process. Android otherwise retains a
        # bound-service record whose receiver belonged to the killed probe process.
        restore_probe = self.original_accessibility()["enabled_accessibility_services"] == SERVICE
        if restore_probe:
            self.shell("settings", "put", "secure", "enabled_accessibility_services", "")
            self.shell("settings", "put", "secure", "accessibility_enabled", 0)
            self.wait(lambda: not any("Bound services:" in line and "label=Renderer probe" in line
                                     for line in self.shell("dumpsys", "accessibility").splitlines()),
                      "Old probe observer did not unbind")
        # Old tree requests must not survive from a preceding fixture/settings task.
        self.shell("am", "force-stop", PROBE)
        time.sleep(.5)
        self.shell("am", "start", "-W", "-f", "0x10008000",
                   "-n", PROBE + "/.ProbeActivity", "--ez", "noSuggestions", str(no_suggestions).lower(), "--es", "field", field,
                   *extras)
        self.wait(lambda: any("topResumedActivity=" in line and PROBE + "/.ProbeActivity" in line
                              for line in self.shell("dumpsys", "activity", "activities").splitlines()),
                  "Disposable probe is not foreground")
        self.reset(no_suggestions, field)
        self.shell("ime", "set", PKG + "/dev.patrickgold.florisboard.FlorisImeService")
        self.tap(540, 480)
        def shown():
            return "mInputShown=true" in self.shell("dumpsys", "input_method")
        if not shown() and "mCurMethod=null" in self.shell("dumpsys", "input_method"):
            # Package replacement can retain a dead service connection even when the
            # selected id is unchanged. Toggle this test device's IME once to rebind it.
            ime = PKG + "/dev.patrickgold.florisboard.FlorisImeService"
            self.shell("ime", "disable", ime)
            self.shell("ime", "enable", ime)
            self.shell("ime", "set", ime)
            self.tap(540, 480)
        self.wait(shown, "IME not shown")
        self.wait(lambda: self.digest()["utf16_units"] == 0, "Probe not empty")
        if restore_probe:
            self.services(SERVICE)

    def reset(self, no_suggestions=False, field="text"):
        self.shell("am", "broadcast", "-a", PROBE + ".RESET", "-p", PROBE,
                   "--ez", "noSuggestions", str(no_suggestions).lower(), "--es", "field", field)
        self.wait(lambda: self.digest()["utf16_units"] == 0, "Probe reset did not publish")

    def digest(self):
        return json.loads(self.adb("exec-out", "run-as", PROBE, "cat", "files/digest.json"))

    def tap(self, x, y):
        self.shell("input", "tap", x, y)

    def hold(self, x, y, duration=400):
        self.shell("input", "swipe", x, y, x, y, duration)

    def swipe_background(self, x, y, duration=700):
        return subprocess.Popen(["adb", "-s", self.serial, "shell", "input", "swipe",
                                 str(x), str(y), str(x), str(y), str(duration)],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
