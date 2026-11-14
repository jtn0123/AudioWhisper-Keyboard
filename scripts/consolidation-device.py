"""Run the opt-in A1/boundary test APK with exact canonical-state restoration. No audio/UI input."""
import argparse
import json
import os
from pathlib import Path
import runpy
import subprocess

ROOT = Path(__file__).resolve().parent.parent
STORES = runpy.run_path(str(ROOT / 'scripts/render-benchmark/store-session.py'))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--baseline', type=Path)
    parser.add_argument('--a1', action='store_true')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    serial = os.environ['ANDROID_SERIAL']
    if serial == 'emulator-5554':
        raise ValueError('Reserved emulator')
    command = ['adb', '-s', serial, 'shell', 'am', 'instrument', '-w']
    if not args.a1:
        command += ['-e', 'boundaries', 'true']
    command += ['com.jtn0123.audiowhisper.keyboard.test/com.jtn0123.audiowhisper.suggest.A1Instrumentation']
    with STORES['fixture_session'](args.output / 'stores'):
        result = subprocess.run(command, capture_output=True, text=True, timeout=300)
        (args.output / 'instrumentation.log').write_text(result.stdout + result.stderr)
        if result.returncode or 'INSTRUMENTATION_CODE: -1' not in result.stdout:
            raise RuntimeError('Device comparison failed; retained instrumentation log')
        if args.a1:
            for key, value in {
                'decoder_sha256': 'daea23ac4c96fcf41d7c8d165faf3fde16e9fbf9ed5c6f5693be19e397e54019',
                'pipeline_sha256': '658c8914602e62d97572366074775389d3f6c49c4941b41d0b90949b253555ba',
            }.items():
                if key + '=' + value not in result.stdout:
                    raise RuntimeError('A1 digest mismatch')
            report = {'a1': 'matched both accepted digests'}
        else:
            prefix = 'INSTRUMENTATION_RESULT: boundaries='
            report = json.loads(next(line[len(prefix):] for line in result.stdout.splitlines() if line.startswith(prefix)))
            if report['reader_reference_comparisons'] != 6_000:
                raise RuntimeError('Incomplete reader reference comparison')
            if args.baseline and report['sha256'] != json.loads(args.baseline.read_text())['sha256']:
                raise RuntimeError('Boundary result digest mismatch')
        (args.output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Passed ' + ('A1 replay' if args.a1 else 'boundary replay and timing') + '; canonical restoration verified')


if __name__ == '__main__':
    main()
