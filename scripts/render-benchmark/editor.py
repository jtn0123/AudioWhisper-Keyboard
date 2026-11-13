"""Fixed editor fixtures: digest/UTF-16 state only. Keep agent-device idle during input."""
import hashlib
import json
from pathlib import Path
import runpy
import sys
import time

api = runpy.run_path(str(Path(__file__).with_name('a11y_device.py')))
panels = runpy.run_path(str(Path(__file__).with_name('panels.py')))
chrome = runpy.run_path(str(Path(__file__).with_name('chrome.py')))
chrome['KEYS'].update({'gestures__delete_key_long_press': 's', 'correction__auto_capitalization': 'b'})
KEYS = ['gestures__delete_key_long_press', 'correction__auto_capitalization']
KEYS += ['keyboard__rust_renderer_prototype', 'suggestion__incognito_mode']
POINTS = {'a': (110, 1900), 'e': (275, 1750), 'h': (650, 1900),
          'i': (810, 1750), 'o': (915, 1750), 'l': (975, 1900), 't': (485, 1750)}


def settled(d):
    old = None
    for _ in range(20):
        value = d.digest()
        if value == old:
            return value
        old = value
        time.sleep(.1)
    raise RuntimeError('Digest/selection did not settle')


def replay(d, renderer):
    d.shell("am", "force-stop", api["PKG"])
    d.renderer(renderer)
    result = {}

    def fresh(field='text'):
        d.probe(field=field)
        d.services(api['SERVICE'])
        ime = api['PKG'] + '/dev.patrickgold.florisboard.FlorisImeService'
        d.shell('ime', 'enable', ime)
        d.shell('ime', 'set', ime)
        d.tap(540, 480)
        d.wait(lambda: d.shell('settings', 'get', 'secure', 'default_input_method').strip() == ime, 'Project IME selection lost')
        tree = d.capture('ready-' + field, lambda ns: any(n.get('text') in ('e', 'E') for n in ns))
        if field != 'caps' and any(n.get('text') == 'E' for n in tree['nodes']):
            d.tap(70, 2060)
            d.capture('lower-' + field, lambda ns: any(n.get('text') == 'e' for n in ns))
        panels['observer_off'](d)

    def type_keys(value):
        for char in value:
            d.tap(*POINTS[char])
            time.sleep(.18)

    def save(name, expected=None):
        value = settled(d)
        if expected is not None:
            assert value['sha256'] == hashlib.sha256(expected.encode()).hexdigest(), name
        result[name] = value
        d.save(name + '-digest.json', value)

    fresh()
    d.shell('am', 'broadcast', '-a', api['PROBE'] + '.EDITOR', '-p', api['PROBE'], '--es', 'fixture', 'emoji')
    time.sleep(.6)
    d.hold(1015, 2060, 400)  # Above long-press delay, below word-repeat delay.
    time.sleep(.5)
    save('emoji-word-delete')
    assert result['emoji-word-delete']['utf16_units'] == 24, 'Expected exactly one word delete'
    for index in range(7):
        d.tap(1015, 2060)
        time.sleep(.2)
        save('emoji-backspace-' + str(index))

    fresh()
    type_keys('teh')
    time.sleep(.8)
    d.tap(540, 2200)
    d.wait(lambda: d.digest()['sha256'] == hashlib.sha256(b'the ').hexdigest(), 'Autocorrect prerequisite')
    d.tap(1015, 2060)
    time.sleep(.4)
    save('autocorrect-revert', 'teh')

    fresh('caps')
    type_keys('hi')
    d.tap(865, 2200)
    d.tap(540, 2200)
    time.sleep(.4)
    type_keys('a')
    save('capital-after-period', 'Hi. A')

    for field in ('url', 'email', 'password'):
        fresh(field)
        type_keys('hello')
        save(field, 'hello')

    fresh()
    type_keys('hel')
    before = settled(d)
    assert before['composing_start'] >= 0, 'External fixture must start while composing'
    result['external-before'] = before
    d.shell('am', 'broadcast', '-a', api['PROBE'] + '.EDITOR', '-p', api['PROBE'], '--es', 'fixture', 'external')
    time.sleep(.5)
    save('external-changed')
    type_keys('a')
    save('external-after-key', 'XaYl')
    return result


def main():
    d = api['Device'](Path(sys.argv[1]))
    original = d.original_accessibility()
    d.save('original-preferences.json', d.prefs())
    try:
        d.mute()
        chrome['set_preferences'](d, {'gestures__delete_key_long_press': 'DELETE_WORD',
                                      'correction__auto_capitalization': True, 'suggestion__incognito_mode': 'FORCE_ON'})
        results = {}
        root = d.output
        for renderer in ('compose', 'rust'):
            d.output = root / renderer
            d.output.mkdir(parents=True, exist_ok=True)
            results[renderer] = replay(d, renderer)
        d.output = root
        d.save('editor.json', results)
        assert results['compose'] == results['rust'], 'Editor renderer state differs'
        if len(sys.argv) > 2:
            assert results == json.loads(Path(sys.argv[2]).read_text()), 'Pre/post editor state differs'
        print('Both renderers matched all fixed editor digests and UTF-16 selection/composing offsets')
    finally:
        d.output = Path(sys.argv[1])
        panels['restore_preferences'](d, d.output / 'original-preferences.json', KEYS)
        d.restore_accessibility(original)


if __name__ == '__main__':
    main()
