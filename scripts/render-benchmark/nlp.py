"""Fixed synthetic NLP traces. Store only digests, counts and explicitly requested screenshots.

Run with python3 -I, the dedicated ANDROID_SERIAL, and keep agent-device idle during input.
The optional --agent captures typing/autocorrect/password states through agent-device.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import runpy
import subprocess
import time
import uuid

api = runpy.run_path(str(Path(__file__).with_name('a11y_device.py')))
chrome = runpy.run_path(str(Path(__file__).with_name('chrome.py')))
panels = runpy.run_path(str(Path(__file__).with_name('panels.py')))
editor = runpy.run_path(str(Path(__file__).with_name('editor.py')))
POINTS = dict(zip('qwertyuiop', [(55+108*i, 1750) for i in range(10)]))
POINTS.update(dict(zip('asdfghjkl', [(110+108*i, 1900) for i in range(9)])))
POINTS.update(dict(zip('zxcvbnm', [(220+108*i, 2060) for i in range(7)])))


def replay(d, renderer, agent):
    d.renderer(renderer)
    results = {}
    def fresh(field='text'):
        d.probe(field=field)
        d.services(api['SERVICE'])
        tree = d.capture('ready-' + field, lambda ns: any(n.get('text') in ('e', 'E') for n in ns))
        if any(n.get('text') == 'E' for n in tree['nodes']):
            d.tap(70, 2060)
        panels['observer_off'](d)
    def type_word(word):
        if os.environ.get('AGENT_DEVICE'):
            for char in word:
                d.hold(*POINTS[char], 20)
            return
        # One shell transaction removes host launch jitter between rapid strokes.
        commands = [f'input swipe {POINTS[c][0]} {POINTS[c][1]} {POINTS[c][0]} {POINTS[c][1]} 20' for c in word]
        d.adb('shell', '; '.join(commands))
    def save(name, expected=None):
        value = editor['settled'](d)
        if expected is not None:
            assert value['sha256'] == hashlib.sha256(expected.encode()).hexdigest(), name
        results[name] = value
        d.save(name + '-digest.json', value)
    def screenshot(name):
        if agent:
            panels['screenshot'](d, agent, name)
        d.services(api['SERVICE'])
        request = uuid.uuid4().hex
        d.shell('am', 'broadcast', '-a', api['PROBE'] + '.IME', '-p', api['PROBE'],
                '--es', 'request', request, '--ez', 'nlpStrip', 'true')
        reply = d.wait(lambda: d.response(request), 'NLP strip observer did not reply')
        assert 'strip' in reply, 'NLP strip capture refused'
        d.save(name + '-strip.json', reply['strip'])
        panels['observer_off'](d)
    fresh()
    type_word('helo')
    time.sleep(.8)
    screenshot('typing')
    d.tap(365, 1619)
    type_word('lo')
    save('fast-pick-mid-word')
    fresh()
    type_word('teh')
    time.sleep(.8)
    d.tap(540, 2200)
    screenshot('after-autocorrect')
    d.shell('am', 'broadcast', '-a', api['PROBE'] + '.EDITOR', '-p', api['PROBE'], '--es', 'fixture', 'cursor-start')
    type_word('a')
    save('autocorrect-cursor-move', 'athe ')
    fresh('password')
    type_word('helo')
    time.sleep(.8)
    screenshot('password')
    save('password', 'helo')
    return results


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('output'); parser.add_argument('--baseline'); parser.add_argument('--agent')
    args = parser.parse_args()
    d = api['Device'](args.output); root = d.output
    original = d.original_accessibility()
    d.save('original-preferences.json', d.prefs())
    try:
        d.mute()
        chrome['set_preferences'](d, {'smartbar__shared_actions_expanded': False, 'suggestion__enabled': True, 'emoji__suggestion_enabled': True,
            'theme__mode': 'ALWAYS_DAY', 'theme__day_theme_id': 'org.florisboard.themes:floris_day',
            'suggestion__incognito_mode': 'FORCE_ON'})
        results = {}
        for renderer in ('compose', 'rust'):
            d.output = root / renderer; d.output.mkdir(parents=True, exist_ok=True)
            results[renderer] = replay(d, renderer, args.agent)
        d.output = root; d.save('nlp.json', results)
        assert results['compose'] == results['rust'], 'NLP renderer digests differ'
        if args.baseline:
            assert results == json.loads(Path(args.baseline).read_text()), 'NLP pre/post digests differ'
        print('Both renderers matched fast-pick, autocorrect/cursor and password fixtures')
    finally:
        d.output = root
        panels['restore_preferences'](d, root / 'original-preferences.json', list(chrome['KEYS']))
        d.restore_accessibility(original)


if __name__ == '__main__':
    main()
