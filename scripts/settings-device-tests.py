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


def accessibility_comparison(reports):
    """Retain every interactive node, including duplicate/empty labels and the editor."""
    result = []
    def groups(screen):
        out = {}
        for node in screen['items']:
            primary = node['label'].split('\n', 1)[0]
            fact = dict(node)
            fact['actions'] = sorted(fact['actions'])
            out.setdefault(primary, []).append(fact)
        for facts in out.values():
            facts.sort(key=lambda fact: json.dumps(fact, sort_keys=True))
        return out
    for left, right in zip(reports['compose']['screens'], reports['rust']['screens'], strict=True):
        assert left['screen'] == right['screen']
        lnodes, rnodes = groups(left), groups(right)
        result.append({'screen': left['screen'], 'compose_count': len(left['items']), 'rust_count': len(right['items']),
            'compose_only': sorted(lnodes.keys()-rnodes.keys()), 'rust_only': sorted(rnodes.keys()-lnodes.keys()),
            'semantic_differences': [{'primary_label': label, 'compose': lnodes.get(label, []), 'rust': rnodes.get(label, [])}
                for label in sorted(lnodes.keys() | rnodes.keys()) if lnodes.get(label) != rnodes.get(label)]})
    return result


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

    def fixture(renderer):
        d.fixture()
        data = d.read_prefs().replace(b'b;other__rust_settings_prototype;false',
                                     b'b;other__rust_settings_prototype;' + str(renderer == 'rust').lower().encode())
        d.private_write(MODULE.PREFS, data)
        d.adb('shell', 'run-as', MODULE.PKG, 'rm', '-f', 'files/settings-device-failure.json')

    def run(mode, renderer='rust', label=None):
        fixture(renderer)
        name = f'{renderer}-{label or mode}'
        result = subprocess.run(['adb', '-s', d.serial, 'shell', 'am', 'instrument', '-w', '-e', 'mode', mode, COMPONENT], check=True, capture_output=True, timeout=300).stdout
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
        if args.mode in ('all', 'ui'):
            reports, files = {}, {}
            for renderer in ('compose', 'rust'):
                reports[renderer] = run('ui', renderer)
                files[renderer] = d.read_prefs()
            identical = files['compose'] == files['rust']
            d.save('preference-parity.json', {'identical_bytes': identical,
                'compose_bytes': len(files['compose']), 'rust_bytes': len(files['rust'])})
            assert identical, 'Complete preference files differ'
            d.save('accessibility-comparison.json', accessibility_comparison(reports))
            accessibility_passed = all(r['accessibility_passed'] and r['system_ime_picker'] for r in reports.values())
            d.save('accessibility-result.json', {'passed': accessibility_passed, 'compose_violations': reports['compose']['accessibility_violations'], 'rust_violations': reports['rust']['accessibility_violations'], 'picker_presented': {k: r['system_ime_picker'] for k,r in reports.items()}, 'real_talkback_phone_session': 'owed'})
        if args.mode == 'metrics':
            run('metrics', 'compose')
        if args.mode in ('all', 'text'):
            run('compose-text', 'compose')
            compose_file = d.read_prefs()
            run('text')
            rust_file = d.read_prefs()
            d.save('text-preference-parity.json', {'identical_bytes': compose_file == rust_file, 'compose_bytes': len(compose_file), 'rust_bytes': len(rust_file)})
            assert compose_file == rust_file, 'Text-only preference files differ'
        if args.mode in ('all', 'death'):
            fixture('rust')
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
                     'settings-device-ui.json', 'settings-device-compose-text.json', 'settings-device-text.json', 'settings-device-crash.json', 'settings-device-failure.json'):
            d.adb('shell', 'run-as', MODULE.PKG, 'rm', '-f', 'files/' + name)
    print('Settings action sequences and exact file restoration passed')
    if not accessibility_passed:
        raise SystemExit('Accessibility assertions failed; see accessibility-result.json (not waived)')


if __name__ == '__main__':
    main()
