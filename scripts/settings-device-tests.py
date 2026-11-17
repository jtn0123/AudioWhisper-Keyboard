#!/usr/bin/env python3
"""Host orchestration for opt-in instrumentation. Stores stay private; receipts contain hashes only."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import time

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location('settings_device', Path(__file__).parent / 'render-benchmark/settings_ui_device.py')
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
COMPONENT = MODULE.PKG + '.test/com.jtn0123.audiowhisper.settingsui.SettingsInstrumentation'


def known_defects(report):
    """Each named known defect once per control, with every audit checkpoint that saw it."""
    found = {}
    for screen in report['screens']:
        for node in screen['items']:
            defect = node.get('known_defect')
            if defect:
                entry = found.setdefault((defect['name'], defect['for']), {**defect, 'occurrences': 0, 'screens': []})
                entry['occurrences'] += 1
                if screen['screen'] not in entry['screens']:
                    entry['screens'].append(screen['screen'])
    return sorted(found.values(), key=lambda e: (e['name'], e['for']))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('baseline', type=Path)
    parser.add_argument('mode', choices=['all', 'ui', 'text', 'death', 'crash', 'metrics'])
    args = parser.parse_args()
    d = MODULE.SettingsDevice(os.environ.get('SETTINGS_RECEIPTS', 'build/stage4-followup3/instrumentation'), args.baseline)
    assert d.adb('emu', 'avd', 'name').decode().splitlines()[0] == 'aw_keyboard_ssd'
    for stream, volume in [(1, 0), (2, 0), (3, 0), (5, 0), (4, 1), (10, 1)]:
        d.adb('shell', 'cmd', 'audio', 'set-volume', str(stream), str(volume))
        assert d.adb('shell', 'cmd', 'audio', 'get-stream-volume', str(stream)).decode().strip().endswith(f'-> {volume}')

    def fixture():
        d.fixture()
        d.adb('shell', 'run-as', MODULE.PKG, 'rm', '-f', 'files/settings-device-failure.json')

    def run(mode, label=None):
        fixture()
        name = f'rust-{label or mode}'
        result = subprocess.run(['adb', '-s', d.serial, 'shell', 'am', 'instrument', '-w', '-e', 'mode', mode, COMPONENT], check=True, capture_output=True, timeout=1200).stdout
        (d.output / f'{name}.log').write_bytes(result)
        if b'PASS ' + mode.encode() not in result:
            detail = d.adb('exec-out', 'run-as', MODULE.PKG, 'cat', 'files/settings-device-failure.json')
            (d.output / f'{name}-failure.json').write_bytes(detail)
            raise RuntimeError(f'{name} failed; see sanitized failure receipt')
        report = json.loads(d.adb('exec-out', 'run-as', MODULE.PKG, 'cat', f'files/settings-device-{mode}.json'))
        d.save(f'{name}.json', report)
        return report

    def appearance(mode):
        # Through the agent session, then closed again before UiAutomation claims the device.
        d.launch('settings/home')
        d.ui('settings', 'appearance', mode)
        d.ui('close')

    accessibility_passed = True
    try:
        # Disconnect only this task's automation session before claiming UiAutomation.
        if os.environ.get('SETTINGS_AGENT_CLOSED') != '1':
            d.ui('close')
        # Rust only. Until the Compose settings screens were removed (docs/compose-removal.md),
        # each stage also ran on Compose and the two had to match; those receipts are in
        # docs/stage4-followup3.md and docs/stage4-followup4.md.
        if args.mode in ('all', 'ui'):
            report = run('ui')
            d.save('preferences.json', {'bytes': len(d.read_prefs())})
            # The rows exercised, in order; group 2 also lists every external launch its rows fired.
            d.save('group-one.json', report['group_one'])
            d.save('group-two.json', report['group_two'])
            # Every library row in order, and the first row's licence dialog text and button.
            d.save('licences.json', report['licences'])
            accessibility_passed = report['accessibility_passed'] and report['system_ime_picker']
            known = known_defects(report)
            assert not known, 'The Rust path has no known accessibility defects to excuse'
            d.save('accessibility-result.json', {'passed': accessibility_passed, 'violations': report['accessibility_violations'],
                'picker_presented': report['system_ime_picker'], 'real_talkback_phone_session': 'owed'})
        if args.mode == 'metrics':
            run('metrics')
        if args.mode in ('all', 'text'):
            # The vocabulary field: its edits saved, its text restored, its EditorInfo.
            d.save('vocabulary-field.json', run('text')['vocabulary'])
        if args.mode in ('all', 'death'):
            fixture()
            for name in ('settings-death-ready', 'settings-death-result.json', 'settings-death-expected.json'):
                d.adb('shell', 'run-as', MODULE.PKG, 'rm', '-f', 'files/' + name)
            run('prepare-death')
            def open_fixture():
                d.ui('open', MODULE.PKG, '--activity', MODULE.PKG + '/com.jtn0123.audiowhisper.settingsui.SettingsDeathActivity', '--no-test-ime')
            open_fixture()
            try:
                def ready():
                    result = subprocess.run(['adb', '-s', d.serial, 'shell', 'run-as', MODULE.PKG, 'test', '-f', 'files/settings-death-ready'], capture_output=True)
                    return result.returncode == 0
                d.wait(ready, 'Background activity did not save its composition', timeout=30)
                expected = json.loads(d.adb('exec-out', 'run-as', MODULE.PKG, 'cat', 'files/settings-death-expected.json'))
                pid = str(expected['pid'])
                d.adb('shell', 'am', 'kill', MODULE.PKG)
                still_alive = pid in subprocess.run(['adb', '-s', d.serial, 'shell', 'pidof', MODULE.PKG], capture_output=True).stdout.decode().split()
                if still_alive:
                    d.adb('shell', 'run-as', MODULE.PKG, 'kill', '-9', pid)
                open_fixture()
                def restored():
                    try:
                        return json.loads(d.adb('exec-out', 'run-as', MODULE.PKG, 'cat', 'files/settings-death-result.json'))
                    except (subprocess.CalledProcessError, json.JSONDecodeError):
                        return None
                result = d.wait(restored, 'No process-death restoration result', timeout=15)
                result['kill'] = 'SIGKILL' if still_alive else 'am kill'
                d.save('process-death.json', result)
                assert result['passed'] and result['restored_bundle']
            finally:
                d.ui('close')
        if args.mode in ('all', 'crash'):
            assert not run('crash')['crash_theme_night']
            # The crash screen follows the system theme: repeat it in dark mode.
            appearance('dark')
            try:
                assert run('crash', label='crash-dark')['crash_theme_night']
            finally:
                appearance('light')
    finally:
        d.cleanup()
        for name in ('settings-death-ready', 'settings-death-result.json', 'settings-death-expected.json', 'settings-death-arm',
                     'settings-device-ui.json', 'settings-device-text.json', 'settings-device-crash.json', 'settings-device-failure.json'):
            d.adb('shell', 'run-as', MODULE.PKG, 'rm', '-f', 'files/' + name)
    print('Settings action sequences and exact file restoration passed')
    if not accessibility_passed:
        raise SystemExit('Accessibility assertions failed; see accessibility-result.json (not waived)')


if __name__ == '__main__':
    main()
