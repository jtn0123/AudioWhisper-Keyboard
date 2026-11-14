"""File-only TTS comparison runner. Requires -PttsFixture=true comparison APKs, owned serial.

python3 -I scripts/tts-proof.py OUTPUT
Install only with scripts/install-dev.sh and its --probe option, before running this script.
The model must already be checksum-verified. Never runs a playback or microphone component.
"""
import json
import os
from pathlib import Path
import runpy
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
os.chdir(root)
out = Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True)
serial = os.environ['ANDROID_SERIAL']
if serial == 'emulator-5554':
    raise ValueError('Reserved emulator')
base = ['adb', '-s', serial]
pkg = 'com.jtn0123.audiowhisper.keyboard'
run = runpy.run_path('scripts/render-benchmark/store-session.py')
with run['fixture_session'](out / 'stores'):
    subprocess.run(base + ['shell', 'run-as', pkg, 'rm', '-f', 'files/tts-comparison.json'], check=True)
    with (out / 'instrumentation.log').open('wb') as log:
        result = subprocess.run(base + ['shell', 'am', 'instrument', '-w', pkg +
            '.test/com.jtn0123.audiowhisper.reader.TtsInstrumentation'], stdout=log, stderr=subprocess.STDOUT)
    assert result.returncode == 0
    assert 'tts_complete=true' in (out / 'instrumentation.log').read_text(), 'TTS instrumentation incomplete'
    raw = subprocess.check_output(base + ['exec-out', 'run-as', pkg, 'cat', 'files/tts-comparison.json'])
    report = json.loads(raw)
    assert report['identical'] and len(report['fixtures']) == 5 and len(report['timings']) == 20
    assert report['stress']['cycles'] == report['stress']['immediate_respeaks'] == 200
    assert report['stress']['heap_plateau']
    (out / 'comparison.json').write_bytes(raw)
    print('TTS: five exact PCM/chunk comparisons, 20 timing trials, 200 cancel/respeak cycles, bounded heap plateau')
# Keep artifacts until the whole proof is recorded. Cleanup is an explicit verified final step.
