"""Dispatch fixtures on the installed APK. Fixed synthetic input; store only digests/offsets.

Keep agent-device idle: ordinary input uses adb and overlapping strokes use the disposable
probe's fixed accessibility gesture. Run with python3 -I and the owned ANDROID_SERIAL.
"""
import hashlib
import json
from pathlib import Path
import runpy
import sys
import time
import uuid

api = runpy.run_path(str(Path(__file__).with_name('a11y_device.py')))
chrome = runpy.run_path(str(Path(__file__).with_name('chrome.py')))
panels = runpy.run_path(str(Path(__file__).with_name('panels.py')))
editor = runpy.run_path(str(Path(__file__).with_name('editor.py')))
KEYS = {'keyboard__hinted_number_row_enabled': 'b', 'keyboard__hinted_symbols_enabled': 'b',
        'gestures__delete_key_long_press': 's', 'keyboard__long_press_delay': 'i',
        'correction__auto_capitalization': 'b'}
chrome['KEYS'].update(KEYS)
POINTS = {'a': (110, 1900), 'b': (650, 2060), 'c': (435, 2060),
          'h': (650, 1900), 'i': (810, 1750)}


def replay(d, renderer):
    d.renderer(renderer)
    results = {}

    def fresh(field='text'):
        d.probe(no_suggestions=True, field=field)
        d.services(api['SERVICE'])
        tree = d.capture('ready-' + field, lambda ns: any(n.get('text') in ('e', 'E') for n in ns))
        if any(n.get('text') == 'E' for n in tree['nodes']):
            d.tap(70, 2060)
        panels['observer_off'](d)

    def text(value):
        for char in value:
            d.hold(*POINTS[char], 35)

    def save(name, expected=None):
        value = editor['settled'](d)
        if expected is not None:
            assert value['sha256'] == hashlib.sha256(expected.encode()).hexdigest(), name
        # Inspect mode after clearing only the disposable editor, without restarting its connection.
        d.shell('am', 'broadcast', '-a', api['PROBE'] + '.EDITOR', '-p', api['PROBE'], '--es', 'fixture', 'clear')
        d.services(api['SERVICE'])
        tree = d.capture(name + '-mode', lambda ns: any(n.get('text') in ('e', 'E') for n in ns))
        value['keyboard_mode'] = 'characters'
        value['uppercase_labels'] = any(n.get('text') == 'E' for n in tree['nodes'])
        results[name] = value
        d.save(name + '-digest.json', value)
        panels['observer_off'](d)

    fresh()
    # Explicit double-tap from unshifted, independent of adb process launch jitter.
    d.adb('shell', 'input swipe 70 2060 70 2060 35; sleep 0.08; input swipe 70 2060 70 2060 35')
    text('ab')
    d.tap(70, 2060)
    text('c')
    save('caps-on-off', 'ABc')

    fresh()
    d.services(api['SERVICE'])
    d.shell('input', 'motionevent', 'DOWN', 275, 1750)
    time.sleep(.6)
    tree = d.capture('accent-open')
    accent = next((n for n in tree.get('nodes', []) if n.get('text') == 'é'), None)
    if accent is None:
        d.shell('input', 'motionevent', 'CANCEL', 275, 1750)
        raise RuntimeError('Accent popup does not expose the fixed é fixture')
    x1, y1, x2, y2 = accent['bounds']
    x, y = (x1+x2)//2, (y1+y2)//2
    d.shell('input', 'motionevent', 'MOVE', x, y)
    d.shell('input', 'motionevent', 'UP', x, y)
    save('accent', 'é')

    fresh()
    d.shell('am', 'broadcast', '-a', api['PROBE'] + '.EDITOR', '-p', api['PROBE'], '--es', 'fixture', 'repeat')
    before = d.digest()
    assert before['utf16_units'] >= 10 and before['selection_start'] == before['utf16_units']
    d.hold(1015, 2060, 2000)
    save('repeat-delete-2s', '')

    fresh()
    d.shell('am', 'broadcast', '-a', api['PROBE'] + '.EDITOR', '-p', api['PROBE'], '--es', 'fixture', 'repeat-long')
    assert d.digest()['utf16_units'] == 1088
    d.hold(1015, 2060, 2000)
    save('repeat-delete-2s-long')
    assert 0 < results['repeat-delete-2s-long']['utf16_units'] < 1088

    fresh()
    d.services(api['SERVICE'])
    request = uuid.uuid4().hex
    d.shell('am', 'broadcast', '-a', api['PROBE'] + '.IME', '-p', api['PROBE'],
            '--es', 'request', request, '--es', 'gesture', 'rolling')
    assert d.wait(lambda: d.response(request), 'Rolling gesture did not complete')['ok']
    save('two-finger-rolling', 'as')

    for field in ('search', 'text'):
        fresh(field)
        text('hi')
        d.hold(1010, 2200, 35)
        time.sleep(.4)
        actual = d.digest()
        assert actual['editor_actions'] == int(field == 'search'), 'IME action count: ' + field
        if field == 'search':
            assert actual['last_editor_action'] == 3
        save('enter-' + field, 'hi' if field == 'search' else 'hi\n')
    return results


def main():
    d = api['Device'](Path(sys.argv[1]))
    original = d.original_accessibility()
    d.save('original-accessibility.json', original)
    d.save('original-preferences.json', d.prefs())
    try:
        d.mute()
        chrome['set_preferences'](d, {'keyboard__hinted_number_row_enabled': False,
            'keyboard__hinted_symbols_enabled': False, 'gestures__delete_key_long_press': 'NO_ACTION',
            'keyboard__long_press_delay': 300, 'correction__auto_capitalization': False})
        root = d.output
        results = {}
        for renderer in ('compose', 'rust'):
            d.output = root / renderer
            d.output.mkdir(parents=True, exist_ok=True)
            results[renderer] = replay(d, renderer)
        d.output = root
        d.save('input.json', results)
        assert results['compose'] == results['rust'], 'Input fixture renderer state differs'
        if len(sys.argv) > 2:
            baseline = json.loads(Path(sys.argv[2]).read_text())
            for renderer, fixtures in baseline.items():
                for name, expected in fixtures.items():
                    assert all(results[renderer][name][key] == value for key, value in expected.items()), 'Pre/post input fixture state differs: ' + name
        print('Both renderers match all input fixture digests, selection, modes and IME actions')
    finally:
        d.output = Path(sys.argv[1])
        panels['restore_preferences'](d, d.output / 'original-preferences.json', list(KEYS))
        d.restore_accessibility(original)


if __name__ == '__main__':
    main()
