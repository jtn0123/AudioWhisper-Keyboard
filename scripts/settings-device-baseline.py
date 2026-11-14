#!/usr/bin/env python3
"""Private baseline for settings tests; stdout/receipts contain no stored values or text."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

PKG = 'com.jtn0123.audiowhisper.keyboard'
FILES = ['jetpref_datastore/florisboard-app-prefs.jetpref', 'files/learned-words.txt', 'files/kept-as-typed.txt']
KEYS = {'secure': ['default_input_method', 'enabled_input_methods', 'enabled_accessibility_services', 'accessibility_enabled'],
        'system': ['font_scale', 'accelerometer_rotation', 'user_rotation'],
        'global': ['window_animation_scale', 'transition_animation_scale', 'animator_duration_scale', 'zen_mode']}
PROPERTIES = ['log.tag.aw-profile', 'log.tag.aw-settings', 'log.tag.aw-compose']
SILENT = {1: 0, 2: 0, 3: 0, 5: 0, 4: 1, 10: 1}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('serial')
    parser.add_argument('directory', type=Path)
    parser.add_argument('--verify', action='store_true')
    args = parser.parse_args()
    if args.serial == 'emulator-5554':
        raise ValueError('Reserved emulator')
    def adb(*arguments):
        return subprocess.run(['adb', '-s', args.serial, *arguments], check=True, capture_output=True, timeout=40).stdout
    assert adb('emu', 'avd', 'name').decode().splitlines()[0] == 'aw_keyboard_ssd'
    state = {'serial': args.serial, 'files': {}, 'properties': {}, 'night': adb('shell', 'cmd', 'uimode', 'night').decode().strip()}
    for namespace, keys in KEYS.items():
        state[namespace] = {key: adb('shell', 'settings', 'get', namespace, key).decode().strip() for key in keys}
    state['properties'] = {key: adb('shell', 'getprop', key).decode().strip() for key in PROPERTIES}
    for index, path in enumerate(FILES):
        data = adb('exec-out', 'run-as', PKG, 'cat', path)
        state['files'][path] = {'file': f'file-{index}', 'sha256': hashlib.sha256(data).hexdigest()}
        if not args.verify:
            args.directory.mkdir(parents=True, exist_ok=True)
            args.directory.chmod(0o700)
            destination = args.directory / f'file-{index}'
            if destination.exists():
                raise ValueError('Baseline already exists; never replace it')
            destination.write_bytes(data)
            destination.chmod(0o600)
    if args.verify:
        original = json.loads((args.directory/'baseline.json').read_text())
        for key in ('serial', 'secure', 'system', 'global', 'night', 'properties', 'files'):
            assert state[key] == original[key], f'Baseline mismatch: {key}'
    else:
        (args.directory/'baseline.json').write_text(json.dumps(state, indent=2)+'\n')
    for stream, volume in SILENT.items():
        if not args.verify:
            adb('shell', 'cmd', 'audio', 'set-volume', str(stream), str(volume))
        assert adb('shell', 'cmd', 'audio', 'get-stream-volume', str(stream)).decode().strip().endswith(f'-> {volume}')
    print('Baseline verified' if args.verify else 'Private baseline captured; all streams muted and verified')


if __name__ == '__main__':
    main()
