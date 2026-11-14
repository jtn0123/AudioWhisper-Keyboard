"""Real muted neural-reader screenshots; application source glyphs stay masked.

Requires the disposable reader field and rendererFixture test APK. No AX snapshot or
UiAutomation gesture runs while the reader service is active. Each renderer gets an exact
canonical-state wrapper. The old/new APK choice belongs to install-dev.sh, outside this runner.
"""
import argparse
import json
import os
from pathlib import Path
import runpy
import subprocess
import time

HERE = Path(__file__).resolve().parent
api = runpy.run_path(str(HERE / 'a11y_device.py'))
panels = runpy.run_path(str(HERE / 'panels.py'))
chrome = runpy.run_path(str(HERE / 'chrome.py'))
stores = runpy.run_path(str(HERE / 'store-session.py'))
PKG = api['PKG']
READER = PKG + '/com.jtn0123.audiowhisper.reader.ReadAloudService'


def operation(d, op):
    d.shell('am', 'broadcast', '-a', 'com.jtn0123.audiowhisper.RENDER_FIXTURE',
            '-p', PKG, '--es', 'op', op)


def reader_state(d):
    d.shell('run-as', PKG, 'rm', '-f', 'files/reader-probe.json')
    def response():
        result = subprocess.run(['adb', '-s', d.serial, 'exec-out', 'run-as', PKG,
                                 'cat', 'files/reader-probe.json'], capture_output=True, text=True)
        if result.returncode == 0:
            try:
                return json.loads(result.stdout)
            except json.JSONDecodeError:
                pass  # exec-out can return zero with cat's missing-file diagnostic on stdout.
        operation(d, 'reader-state')
        return None
    return d.wait(response, 'Reader state probe absent')


def replay(output, renderer, agent):
    d = api['Device'](output)
    chrome['set_preferences'](d, {'keyboard__rust_renderer_prototype': renderer == 'rust',
        'theme__mode': 'ALWAYS_DAY', 'theme__day_theme_id': 'org.florisboard.themes:floris_day',
        'suggestion__enabled': False, 'emoji__suggestion_enabled': False})
    d.mute()
    runner = PKG + '.test/com.jtn0123.audiowhisper.render.FixtureInstrumentation'
    assert runner in d.shell('pm', 'list', 'instrumentation', PKG), 'Install the rendererFixture test APK'
    log_path = d.output / 'controller.log'
    with log_path.open('wb') as log:
        controller = subprocess.Popen(['adb', '-s', d.serial, 'shell', 'am', 'instrument', '-w',
            PKG + '.test/com.jtn0123.audiowhisper.render.FixtureInstrumentation'], stdout=log, stderr=subprocess.STDOUT)
        try:
            assert reader_state(d)['status'] == 'IDLE'
            d.probe(True, field='reader')
            d.services(api['SERVICE'])
            tree = d.capture('ready', lambda ns: any(n.get('text') in ('e', 'E') for n in ns))
            point = panels['center'](panels['named'](tree['nodes'], 'Read aloud'))
            panels['observer_off'](d)
            d.shell('settings', 'put', 'secure', 'enabled_accessibility_services', READER)
            d.shell('settings', 'put', 'secure', 'accessibility_enabled', 1)
            d.wait(lambda: 'ReadAloudService' in d.shell('dumpsys', 'accessibility'), 'Reader service absent')
            d.hold(*point, 35)
            def playing():
                state = reader_state(d)
                return state if state['status'] == 'READING' and state.get('neural') and state.get('playback_head_frames', 0) > 0 else None
            active = d.wait(playing, 'Muted neural playback did not advance', timeout=45)
            # No snapshots: masked application glyphs keep read-aloud text out of images.
            subprocess.run([agent, 'screenshot', str(d.output / 'reading.png')],
                           stdout=subprocess.DEVNULL, check=True)
            after_capture = reader_state(d)
            d.save('capture-state.json', {'active': active, 'after_capture': after_capture})
            assert after_capture['status'] == 'READING', 'Reader fixture ended before the stop gesture'
            # Stop through the real floating pill shown in the screenshot (Pixel 8 geometry).
            d.hold(540, 135, 35)
            d.wait(lambda: reader_state(d)['status'] == 'IDLE', 'Reader did not stop')
            time.sleep(.3)
            subprocess.run([agent, 'screenshot', str(d.output / 'stopped.png')],
                           stdout=subprocess.DEVNULL, check=True)
            assert d.digest()['utf16_units'] == 0
            d.save('reader.json', {'active': active, 'stopped': reader_state(d),
                'editor_empty': True, 'application_glyphs_masked': True, 'muted': True})
        finally:
            operation(d, 'finish')
            controller.wait(timeout=45)
            assert 'fixture_complete=true' in log_path.read_text(), 'Reader controller incomplete'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    agent = os.environ['AGENT_DEVICE']
    for renderer in ('compose', 'rust'):
        with stores['fixture_session'](args.output / (renderer + '-stores')):
            replay(args.output / renderer, renderer, agent)
    print('Both renderers: muted real-model playback advanced; stop/highlight screenshots captured')


if __name__ == '__main__':
    main()
