"""Settings prototype device transport. UI always uses the T3-bound agent-device command.

Only empty-field trees may be saved. Typed field checks retain hashes/counts only.
"""
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import time

PKG = 'com.jtn0123.audiowhisper.keyboard'
PREFS = 'jetpref_datastore/florisboard-app-prefs.jetpref'
IME = PKG + '/dev.patrickgold.florisboard.FlorisImeService'
ROOT = Path(__file__).resolve().parents[2]


class SettingsDevice:
    def __init__(self, output, baseline):
        self.output = Path(output).resolve()
        self.output.mkdir(parents=True, exist_ok=True)
        self.baseline = Path(baseline).resolve()
        self.original = json.loads((self.baseline / 'baseline.json').read_text())
        self.serial = os.environ['ANDROID_SERIAL']
        self.agent = os.environ['AGENT_DEVICE']
        if self.serial == 'emulator-5554' or self.original['serial'] != self.serial:
            raise ValueError('This proof requires the owned aw_keyboard_ssd runtime')
        if self.adb('emu', 'avd', 'name').decode().splitlines()[0] != 'aw_keyboard_ssd':
            raise ValueError('This proof requires the aw_keyboard_ssd AVD')

    def adb(self, *args, input=None):
        return subprocess.run(['adb', '-s', self.serial, *map(str, args)], input=input,
                              capture_output=True, check=True, timeout=40).stdout

    def ui(self, *args, json_result=False):
        result = subprocess.run([self.agent, *map(str, args)], capture_output=True, text=True, timeout=60)
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return json.loads(result.stdout)['data'] if json_result else None

    def save(self, name, value):
        (self.output / name).write_text(json.dumps(value, indent=2) + '\n')

    def private_write(self, path, data):
        command = 'run-as ' + PKG + ' sh -c ' + shlex.quote('cat > ' + path)
        self.adb('shell', command, input=data)

    def read_prefs(self):
        return self.adb('exec-out', 'run-as', PKG, 'cat', PREFS)

    def prefs(self):
        return {p[1]: p[2] for line in self.read_prefs().decode().splitlines()
                if len(p := line.split(';', 2)) == 3}

    def wait(self, check, message, timeout=12):
        until = time.monotonic() + timeout
        while time.monotonic() < until:
            value = check()
            if value:
                return value
            time.sleep(.1)
        raise RuntimeError(message)

    def pid(self):
        return self.adb('shell', 'pidof ' + PKG + ' || true').decode().strip()

    def restore_files(self):
        # The bytes go in while the app process is dead, so the next process loads them. A
        # process alive across the write keeps the old values in memory, and Jetpref's next
        # write (any preference, the whole file) puts them back over the restored bytes.
        originals = {}
        for path, entry in self.original['files'].items():
            originals[path] = (self.baseline / entry['file']).read_bytes()
            if hashlib.sha256(originals[path]).hexdigest() != entry['sha256']:
                raise RuntimeError('Original state artifact changed')
        for _ in range(5):
            self.adb('shell', 'am', 'force-stop', PKG)
            for path, data in originals.items():
                self.private_write(path, data)
            if not self.pid():
                break
        else:
            raise RuntimeError('The app restarted during every file restoration')
        # A stopped selected IME can be replaced by Android, so the original is bound again. Its
        # process now starts from the restored bytes; its startup writes, if any, are let land
        # before the bytes are verified, so a startup that changes them fails here.
        ime = self.original['secure']['default_input_method']
        self.adb('shell', 'ime', 'set', ime)
        if ime == IME:
            self.wait(self.pid, 'The original IME did not start')
        self.wait(self.files_settled, 'The restored files kept changing', timeout=20)
        self.verify_files()

    def files_settled(self):
        before = self.hashes()
        time.sleep(1.5)
        return before == self.hashes()

    def hashes(self):
        return {path: self.adb('shell', 'run-as', PKG, 'sha256sum', path).decode().split()[0]
                for path in self.original['files']}

    def verify_files(self):
        actual = self.hashes()
        expected = {path: entry['sha256'] for path, entry in self.original['files'].items()}
        if actual != expected:
            raise RuntimeError('Exact original file restoration failed')
        return actual

    def fixture(self):
        self.restore_files()
        source = (self.baseline / self.original['files'][PREFS]['file']).read_bytes()
        lines = source.splitlines(keepends=True)
        # Older builds stored the removed settings switch; the app's migration drops it on load.
        lines = [line for line in lines if b';other__rust_settings_prototype;' not in line]
        if lines and not lines[-1].endswith(b'\n'):
            lines[-1] += b'\n'
        self.private_write(PREFS, b''.join(lines))

    def launch(self, route):
        self.ui('open', PKG, 'ui://florisboard/' + route, '--no-test-ime',
                '--launch-args', '-a', '--launch-args', 'android.intent.action.VIEW',
                '--launch-args', '-c', '--launch-args', 'android.intent.category.BROWSABLE')

    def tree(self):
        return self.ui('snapshot', '--json', json_result=True)

    def app_nodes(self, tree=None):
        return [node for node in (tree or self.tree())['nodes'] if node.get('bundleId') == PKG
                and node.get('visibleToUser', True)]

    @staticmethod
    def label(node):
        return node.get('label') or node.get('contentDescription') or ''

    def find(self, label, tree=None):
        return next((n for n in self.app_nodes(tree) if self.label(n) == label or self.label(n).startswith(label + ',')), None)

    def press(self, label, scroll=False):
        for attempt in range(12 if scroll else 1):
            node = self.find(label)
            if node:
                self.ui('press', 'role=' + node['kind'] + ' label=' + json.dumps(self.label(node)))
                return
            if scroll:
                # Slow enough to leave no fling: a fling's end depends on where its batched events
                # fall between frames, and a next gesture can catch one still running.
                self.ui('scroll', 'down', 480, '--duration-ms', 1200)
        raise RuntimeError('Settings target absent: ' + label)

    def prototype(self, renderer):
        """Opens Home. Rust is the only settings renderer: the switch and the Compose screens it
        chose were removed (docs/compose-removal.md)."""
        if renderer != 'rust':
            raise SystemExit('The Compose settings screens were removed; only "rust" remains')
        self.launch('settings/home')
        self.wait(lambda: self.find('Keyboard'), 'Home did not appear')

    def screenshot(self, filename):
        path = self.output / filename
        self.ui('screenshot', path)
        return str(path)

    def empty_tree(self, filename):
        tree = self.tree()
        for node in self.app_nodes(tree):
            if node.get('editable') and not node.get('hintShowing', False):
                label = self.label(node)
                if label not in ('', 'Try out your setup'):
                    raise RuntimeError('Refusing to persist a nonempty text-field tree')
        self.save(filename, tree)
        return tree

    def field_digest(self):
        nodes = [n for n in self.app_nodes() if n.get('editable')]
        if len(nodes) != 1:
            raise RuntimeError('Expected one editable preview endpoint')
        node = nodes[0]
        value = node.get('value', self.label(node))
        if node.get('hintShowing') or value == 'Try out your setup':
            value = ''
        return {'utf16_units': len(value.encode('utf-16-le', errors='surrogatepass')) // 2,
                'sha256': hashlib.sha256(value.encode('utf-8', errors='surrogatepass')).hexdigest()}

    def slider_minimum(self):
        nodes = [n for n in self.app_nodes() if n.get('type') == 'android.widget.SeekBar']
        if not nodes:
            raise RuntimeError('No real slider semantic node')
        box = nodes[0]['rect']
        self.ui('press', box['x'] + 1, box['y'] + box['height'] // 2)

    def stock_ime(self):
        installed = self.adb('shell', 'ime', 'list', '-a', '-s').decode().splitlines()
        system = set(self.adb('shell', 'pm', 'list', 'packages', '-s').decode().splitlines())
        allowed = ('com.google.android.inputmethod.latin', 'com.android.inputmethod.latin')
        candidate = next((ime.strip() for ime in installed
                          if ime.split('/')[0] in allowed and 'package:' + ime.split('/')[0] in system), None)
        if candidate is None:
            raise RuntimeError('No stock system keyboard is available')
        self.adb('shell', 'ime', 'set', candidate)
        selected = self.adb('shell', 'settings', 'get', 'secure', 'default_input_method').decode().strip()
        if selected != candidate:
            raise RuntimeError('Stock keyboard selection was not applied')
        return candidate

    def restore_ime(self):
        self.adb('shell', 'ime', 'set', IME)
        selected = self.adb('shell', 'settings', 'get', 'secure', 'default_input_method').decode().strip()
        if selected != IME:
            raise RuntimeError('Project keyboard selection was not restored')

    def file_receipt(self, name):
        data = self.read_prefs()
        path = self.output / (name + '.jetpref')
        path.write_bytes(data)
        path.chmod(0o600)
        return {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'path': str(path)}

    def cleanup(self):
        # Restore settings via the platform state interface only when changed; files are
        # restored last, and the IME then starts from them (see restore_files).
        for key in ('enabled_accessibility_services', 'accessibility_enabled'):
            original = self.original['secure'][key]
            current = self.adb('shell', 'settings', 'get', 'secure', key).decode().strip()
            if current != original:
                args = ['settings', 'delete' if original == 'null' else 'put', 'secure', key]
                if original != 'null':
                    args.append(original)
                self.adb('shell', *args)
        self.restore_files()
        for _ in range(3):
            time.sleep(.3)
            self.verify_files()
        receipt = {'files': self.verify_files(), 'secure': {key: self.adb('shell', 'settings', 'get', 'secure', key).decode().strip()
                   for key in self.original['secure']},
                   'font_scale': self.adb('shell', 'settings', 'get', 'system', 'font_scale').decode().strip()}
        if receipt['secure'] != self.original['secure'] or receipt['font_scale'] != self.original['system']['font_scale']:
            raise RuntimeError('Original Android settings were not restored')
        self.save('cleanup.json', receipt)
        return receipt
