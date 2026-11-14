"""Replay the 17 existing extra records with canonical state and hash-only clipboard evidence.

Requires AGENT_DEVICE, ANDROID_SERIAL, DEVICE_BASELINE and rendererFixture APKs installed
through install-dev.sh. All outputs must be under ignored build/.
"""
import argparse
from contextlib import contextmanager
import json
import os
from pathlib import Path
import runpy
import subprocess
import time

HERE = Path(__file__).resolve().parent
stores = runpy.run_path(str(HERE / 'store-session.py'))
api = runpy.run_path(str(HERE / 'a11y_device.py'))
panels = runpy.run_path(str(HERE / 'panels.py'))
chrome = runpy.run_path(str(HERE / 'chrome.py'))
clipboard = runpy.run_path(str(HERE / 'clipboard-gif.py'))
learning = runpy.run_path(str(HERE / 'nlp-learning.py'))
PKG, PROBE = api['PKG'], api['PROBE']


def operation(d, op):
    d.shell('am', 'broadcast', '-a', 'com.jtn0123.audiowhisper.RENDER_FIXTURE',
            '-p', PKG, '--es', 'op', op)


def exists(d, path):
    return subprocess.run(['adb', '-s', d.serial, 'shell', 'run-as', PKG, 'test', '-f', path],
                          capture_output=True).returncode == 0


@contextmanager
def room_baseline(d, renderer):
    """Keep this fresh device's empty baseline on-device, never export database contents."""
    held = 'files/acceptance-room-held-' + renderer
    names = ['clipboard_history' + suffix for suffix in ('', '-wal', '-shm')]
    d.shell('am', 'force-stop', PKG)
    d.shell('run-as', PKG, 'mkdir', '-p', held)
    before = {}
    for name in names:
        path = 'databases/' + name
        before[name] = d.shell('run-as', PKG, 'sha256sum', path).split()[0] if exists(d, path) else None
        if before[name]:
            d.shell('run-as', PKG, 'cp', path, held + '/' + name)
    d.shell('ime', 'set', PKG + '/dev.patrickgold.florisboard.FlorisImeService')
    try:
        yield
    finally:
        d.shell('am', 'force-stop', PKG)
        after = {}
        for name in names:
            path = 'databases/' + name
            if before[name]:
                d.shell('run-as', PKG, 'cp', held + '/' + name, path)
            else:
                d.shell('run-as', PKG, 'rm', '-f', path)
            after[name] = d.shell('run-as', PKG, 'sha256sum', path).split()[0] if exists(d, path) else None
        d.save('room-restoration.json', {'before': before, 'after': after, 'identical': before == after})
        assert before == after, 'Room baseline hashes differ'
        d.shell('run-as', PKG, 'rm', '-r', held)


def controller_replay(output, renderer):
    d = panels['Device'](output)
    chrome['KEYS']['clipboard__history_enabled'] = 'b'
    chrome['set_preferences'](d, {'keyboard__rust_renderer_prototype': renderer == 'rust',
        'clipboard__history_enabled': True, 'suggestion__enabled': False, 'emoji__suggestion_enabled': False})
    with room_baseline(d, renderer):
        log_path = d.output / 'controller.log'
        with log_path.open('wb') as log:
            controller = subprocess.Popen(['adb', '-s', d.serial, 'shell', 'am', 'instrument', '-w',
                PKG + '.test/com.jtn0123.audiowhisper.render.FixtureInstrumentation'], stdout=log, stderr=subprocess.STDOUT)
            clipboard_saved = False
            try:
                learning['state'](d)
                assert controller.poll() is None, 'Fixture controller exited'
                assert clipboard['history'](d) == [], 'Dedicated clipboard baseline must be empty'
                d.probe(True)
                d.shell('am', 'broadcast', '-a', PROBE + '.CLIP_STATE', '-p', PROBE, '--es', 'operation', 'snapshot')
                clipboard_saved = True
                d.probe(True, field='batch')
                before = d.digest()
                assert before['batch_depth'] == 0
                d.shell('run-as', PKG, 'rm', '-f', 'files/render-batch-fixture.json')
                operation(d, 'invalid-finalize')
                def reply():
                    result = subprocess.run(['adb', '-s', d.serial, 'exec-out', 'run-as', PKG,
                        'cat', 'files/render-batch-fixture.json'], capture_output=True, text=True)
                    return json.loads(result.stdout) if result.returncode == 0 else None
                assert d.wait(reply, 'Invalid finalize response absent')['accepted'] is False
                time.sleep(.3)
                actual = d.digest()
                assert actual['batch_depth'] == 0
                assert all(actual[key] == before[key] for key in ('sha256', 'selection_start', 'selection_end'))
                d.save('batch.json', actual)
                d.probe(True)
                clipboard['search'](d, renderer)
                clip_output = d.output.parent / ('clipboard-' + renderer)
                d.output = clip_output
                d.output.mkdir(parents=True, exist_ok=True)
                clipboard['workflow'](d, renderer)
                d.output = output
            finally:
                if clipboard_saved:
                    d.shell('am', 'broadcast', '-a', PROBE + '.CLIP_STATE', '-p', PROBE, '--es', 'operation', 'restore')
                operation(d, 'finish')
                controller.wait(timeout=45)
                assert 'fixture_complete=true' in log_path.read_text(), 'Fixture controller incomplete'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--baseline', required=True, type=Path)
    args = parser.parse_args()
    os.environ['AGENT_DEVICE']
    os.environ['DEVICE_BASELINE']
    os.environ['HASH_ONLY_CLIPBOARD'] = '1'
    args.output.mkdir(parents=True, exist_ok=False)
    for renderer in ('compose', 'rust'):
        with stores['fixture_session'](args.output / ('panels-' + renderer + '-stores')):
            with (args.output / ('panels-' + renderer + '.log')).open('wb') as log:
                subprocess.run(['python3', '-I', str(HERE / 'panels.py'), renderer,
                    str(args.output / ('panels-' + renderer))], stdout=log, stderr=subprocess.STDOUT, check=True)
    with stores['fixture_session'](args.output / 'chrome-stores'):
        with (args.output / 'chrome.log').open('wb') as log:
            subprocess.run(['python3', '-I', str(HERE / 'chrome.py'), 'workflow', str(args.output / 'chrome')],
                           stdout=log, stderr=subprocess.STDOUT, check=True)
    for renderer in ('compose', 'rust'):
        with stores['fixture_session'](args.output / ('controller-' + renderer + '-stores')):
            controller_replay(args.output / ('controller-' + renderer), renderer)
    manifest = json.loads((args.baseline / 'comparison.json').read_text())['files']
    assert len(manifest) == 17
    for name in manifest:
        assert json.loads((args.output / name).read_text()) == json.loads((args.baseline / name).read_text()), name
    (args.output / 'comparison.json').write_text(json.dumps({'match': True, 'files': manifest}, indent=2) + '\n')
    print('All 17 extra records match; canonical preferences/stores and Room hashes restored')


if __name__ == '__main__':
    main()
