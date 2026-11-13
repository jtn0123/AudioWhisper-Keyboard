"""Fixed synthetic learning/unlearning through the live manager's test-APK transport.

Run python3 -I with rendererFixture instrumentation active; only digests/counts are retained.
"""
import hashlib
import json
from pathlib import Path
import runpy
import subprocess
import sys
import time

nlp = runpy.run_path(str(Path(__file__).with_name('nlp.py')))
api, chrome, panels = (nlp[k] for k in ('api', 'chrome', 'panels'))
chrome['KEYS']['suggestion__learn_from_typing'] = 'b'


def stored(d):
    result = subprocess.run(['adb', '-s', d.serial, 'exec-out', 'run-as', api['PKG'], 'cat', 'files/learned-words.txt'], capture_output=True)
    return hashlib.sha256(result.stdout if result.returncode == 0 else b'').hexdigest()


def state(d):
    d.shell('run-as', api['PKG'], 'rm', '-f', 'files/nlp-state-fixture.json')
    def response():
        result = subprocess.run(['adb', '-s', d.serial, 'exec-out', 'run-as', api['PKG'], 'cat', 'files/nlp-state-fixture.json'], capture_output=True)
        try:
            if result.returncode == 0 and result.stdout:
                return json.loads(result.stdout)
        except json.JSONDecodeError:
            pass
        d.shell('am', 'broadcast', '-a', 'com.jtn0123.audiowhisper.RENDER_FIXTURE', '-p', api['PKG'], '--es', 'op', 'nlp-state')
        return None
    return d.wait(response, 'Test APK did not return NLP counts', timeout=25)


def main():
    d = api['Device'](sys.argv[1]); root = d.output
    original = d.original_accessibility(); d.save('original-preferences.json', d.prefs())
    results = {}
    try:
        d.mute()
        for renderer in ('compose', 'rust'):
            # Restarting private preference writes also stops instrumentation, so configure once
            # before starting the test controller for this renderer.
            chrome['set_preferences'](d, {'keyboard__rust_renderer_prototype': renderer == 'rust',
                'suggestion__enabled': True, 'emoji__suggestion_enabled': True,
                'suggestion__learn_from_typing': True, 'suggestion__incognito_mode': 'FORCE_OFF'})
            log = (root / (renderer + '-controller.log')).open('wb')
            controller = subprocess.Popen(['adb', '-s', d.serial, 'shell', 'am', 'instrument', '-w',
                api['PKG'] + '.test/com.jtn0123.audiowhisper.render.FixtureInstrumentation'], stdout=log, stderr=subprocess.STDOUT)
            try:
                state(d)  # Receiver handshake, rather than racing a fixed startup delay.
                assert controller.poll() is None, 'Fixture controller exited during startup'
                d.probe(); d.services(api['SERVICE'])
                tree = d.capture(renderer + '-learn-ready', lambda ns: any(n.get('text') in ('e', 'E') for n in ns))
                if any(n.get('text') == 'E' for n in tree['nodes']): d.tap(70, 2060)
                panels['observer_off'](d)
                before = stored(d)
                for c in 'zorblaxq': d.hold(*nlp['POINTS'][c], 35)
                d.tap(540, 2200)
                time.sleep(6)
                learned = stored(d)
                assert learned != before, 'Synthetic word was not learned'
                d.shell('am', 'broadcast', '-a', 'com.jtn0123.audiowhisper.RENDER_FIXTURE', '-p', api['PKG'], '--es', 'op', 'nlp-unlearn')
                time.sleep(6)
                unlearned = stored(d)
                assert unlearned == before, 'Unlearning did not restore the prior store'
                d.probe(field='password')
                for c in 'helo': d.hold(*nlp['POINTS'][c], 35)
                time.sleep(.8)
                counts = state(d)
                assert counts['candidates'] == 0, 'Password candidates exposed'
                results[renderer] = {'before': before, 'learned': learned, 'unlearned': unlearned, 'password': counts}
            finally:
                d.shell('am', 'broadcast', '-a', 'com.jtn0123.audiowhisper.RENDER_FIXTURE', '-p', api['PKG'], '--es', 'op', 'finish')
                controller.wait(timeout=45); log.close()
                assert 'fixture_complete=true' in (root / (renderer + '-controller.log')).read_text(), 'Fixture controller did not complete'
            # Incognito changes require a fresh process, with no private state retained from learning.
            chrome['set_preferences'](d, {'suggestion__incognito_mode': 'FORCE_ON'})
            d.probe(); d.services(api['SERVICE'])
            tree = d.capture(renderer + '-private-ready', lambda ns: any(n.get('text') in ('e', 'E') for n in ns))
            if any(n.get('text') == 'E' for n in tree['nodes']): d.tap(70, 2060)
            panels['observer_off'](d)
            before = stored(d)
            for c in 'glorpzaxq': d.hold(*nlp['POINTS'][c], 35)
            d.tap(540, 2200); time.sleep(6)
            assert stored(d) == before, 'Incognito learned a word'
            results[renderer]['incognito_unchanged'] = True
        d.save('learning.json', results)
        assert results['compose'] == results['rust'], 'Learning renderer effects differ'
        print('Both renderers: learned/unlearned stores restored, incognito unchanged, password candidates=0')
    finally:
        panels['restore_preferences'](d, root / 'original-preferences.json', list(chrome['KEYS']))
        d.restore_accessibility(original)


if __name__ == '__main__': main()
