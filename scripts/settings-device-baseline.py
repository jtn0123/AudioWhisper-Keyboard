#!/usr/bin/env python3
"""Private baseline of the aw_keyboard_ssd emulator for the settings tests.

The baseline is the keyboard app's preference, learned-word and kept-as-typed files plus the
Android settings the tests touch: input methods, accessibility, font scale, rotation,
animations, Do Not Disturb, dark mode and the keyboard's log tags. Output and receipts name
which parts differ, never a stored value or any text.

  SERIAL DIR            capture: save the baseline to DIR, which must not hold one yet, then
                        mute every stream. A baseline is never replaced.
  SERIAL DIR --verify   check only: fail, naming the parts, if the device differs from DIR or a
                        stream is not muted. Changes nothing, so it can't fix a difference.
  SERIAL DIR --restore  put back whatever differs from DIR, mute every stream, then check as
                        --verify does. Leave any text field first: the app is stopped when its
                        files are put back.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shlex
import subprocess
import time

PKG = 'com.jtn0123.audiowhisper.keyboard'
IME = PKG + '/dev.patrickgold.florisboard.FlorisImeService'
FILES = ['jetpref_datastore/florisboard-app-prefs.jetpref', 'files/learned-words.txt', 'files/kept-as-typed.txt']
KEYS = {'secure': ['default_input_method', 'enabled_input_methods', 'enabled_accessibility_services', 'accessibility_enabled'],
        'system': ['font_scale', 'accelerometer_rotation', 'user_rotation'],
        'global': ['window_animation_scale', 'transition_animation_scale', 'animator_duration_scale', 'zen_mode']}
PROPERTIES = ['log.tag.aw-profile', 'log.tag.aw-settings', 'log.tag.aw-compose']
SILENT = {1: 0, 2: 0, 3: 0, 5: 0, 4: 1, 10: 1}
COMPARED = ('serial', 'secure', 'system', 'global', 'night', 'properties', 'files')


class Device:
    def __init__(self, serial):
        if serial == 'emulator-5554':
            raise ValueError('Reserved emulator')
        self.serial = serial
        if self.adb('emu', 'avd', 'name').decode().splitlines()[0].strip() != 'aw_keyboard_ssd':
            raise ValueError('This baseline belongs to the aw_keyboard_ssd AVD')

    def adb(self, *arguments, input=None):
        return subprocess.run(['adb', '-s', self.serial, *arguments], input=input,
                              check=True, capture_output=True, timeout=40).stdout

    def shell(self, *words, input=None):
        """One quoted command line, so a value holding `;` or nothing at all arrives whole."""
        return self.adb('shell', shlex.join(words), input=input).decode().strip()

    def read(self, path):
        return self.adb('exec-out', 'run-as', PKG, 'cat', path)

    def write(self, path, data):
        self.shell('run-as', PKG, 'sh', '-c', 'cat > ' + shlex.quote(path), input=data)

    def pid(self):
        return self.adb('shell', f'pidof {PKG} || true').decode().strip()


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def snapshot(device):
    """The device's side of the baseline, and the bytes of each app file."""
    data = {path: device.read(path) for path in FILES}
    state = {'serial': device.serial,
             'files': {path: {'file': f'file-{index}', 'sha256': sha256(data[path])} for index, path in enumerate(FILES)},
             'properties': {key: device.shell('getprop', key) for key in PROPERTIES},
             'night': device.shell('cmd', 'uimode', 'night')}
    for namespace, keys in KEYS.items():
        state[namespace] = {key: device.shell('settings', 'get', namespace, key) for key in keys}
    return state, data


def mute(device, apply):
    for stream, volume in SILENT.items():
        if apply:
            device.shell('cmd', 'audio', 'set-volume', str(stream), str(volume))
        if not device.shell('cmd', 'audio', 'get-stream-volume', str(stream)).endswith(f'-> {volume}'):
            raise RuntimeError(f'Stream {stream} is not at volume {volume}')


def capture(device, directory):
    state, data = snapshot(device)
    targets = [directory / 'baseline.json'] + [directory / entry['file'] for entry in state['files'].values()]
    if any(target.exists() for target in targets):
        raise ValueError('Baseline already exists; never replace it')
    directory.mkdir(parents=True, exist_ok=True)
    directory.chmod(0o700)
    for path, entry in state['files'].items():
        destination = directory / entry['file']
        destination.write_bytes(data[path])
        destination.chmod(0o600)
    (directory / 'baseline.json').write_text(json.dumps(state, indent=2) + '\n')
    mute(device, apply=True)
    return 'Private baseline captured; all streams muted and verified'


def verify(device, original):
    state, _ = snapshot(device)
    differing = [key for key in COMPARED if state[key] != original[key]]
    if differing:
        raise RuntimeError('Baseline mismatch: ' + ', '.join(differing))
    mute(device, apply=False)
    return 'Baseline verified'


def wait(check, message, timeout):
    until = time.monotonic() + timeout
    while time.monotonic() < until:
        if check():
            return
        time.sleep(.2)
    raise RuntimeError(message)


def restore_settings(device, original):
    """Each Android setting back as it was, all but the keyboard, which goes last."""
    for namespace in KEYS:
        for key, value in original[namespace].items():
            if key == 'default_input_method' or device.shell('settings', 'get', namespace, key) == value:
                continue
            if value == 'null':
                device.shell('settings', 'delete', namespace, key)
            else:
                device.shell('settings', 'put', namespace, key, value)
    if device.shell('cmd', 'uimode', 'night') != original['night']:
        device.shell('cmd', 'uimode', 'night', original['night'].rsplit(':', 1)[1].strip())
    for key, value in original['properties'].items():
        if device.shell('getprop', key) != value:
            device.shell('setprop', key, value)


def restore_files(device, original, directory):
    """The app's files back, written while its process is dead.

    A process alive across the write keeps the old values in memory, and Jetpref's next write
    (any preference, the whole file) puts them back over the restored bytes; SettingsDevice
    in render-benchmark/settings_ui_device.py restores the same way for the same reason.
    """
    saved = {}
    for path, entry in original['files'].items():
        saved[path] = (directory / entry['file']).read_bytes()
        if sha256(saved[path]) != entry['sha256']:
            raise RuntimeError('The saved copy of an app file changed')
    if all(sha256(device.read(path)) == entry['sha256'] for path, entry in original['files'].items()):
        return
    for _ in range(5):
        device.shell('am', 'force-stop', PKG)
        for path, data in saved.items():
            device.write(path, data)
        if not device.pid():
            return
    raise RuntimeError('The app restarted during every file restoration')


def restore(device, directory, original):
    if original['serial'] != device.serial:
        raise ValueError('This baseline was captured on another device')
    restore_settings(device, original)
    restore_files(device, original, directory)
    # Bound again even when unchanged, as Android can replace a stopped keyboard. Its process
    # then starts from the restored files; any startup writes land before they are checked.
    ime = original['secure']['default_input_method']
    device.shell('ime', 'set', ime)
    if ime == IME:
        wait(device.pid, 'The original keyboard did not start', timeout=12)
    def settled():
        before = [device.read(path) for path in FILES]
        time.sleep(1.5)
        return before == [device.read(path) for path in FILES]
    wait(settled, 'The restored files kept changing', timeout=20)
    mute(device, apply=True)
    return verify(device, original).replace('verified', 'restored and verified')


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('serial', metavar='SERIAL', help='the aw_keyboard_ssd emulator, never emulator-5554')
    parser.add_argument('directory', metavar='DIR', type=Path, help='the private baseline directory, under .device-backup/')
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--verify', action='store_true', help='check the device against DIR; changes nothing')
    mode.add_argument('--restore', action='store_true', help='put back what differs from DIR, then check')
    args = parser.parse_args()
    device = Device(args.serial)
    if not (args.verify or args.restore):
        print(capture(device, args.directory))
        return
    original = json.loads((args.directory / 'baseline.json').read_text())
    print(restore(device, args.directory, original) if args.restore else verify(device, original))


if __name__ == '__main__':
    main()
