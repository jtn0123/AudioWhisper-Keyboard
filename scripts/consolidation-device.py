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
    parser.add_argument('--runner', choices=('a1', 'renderer'), default='a1')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    serial = os.environ['ANDROID_SERIAL']
    if serial == 'emulator-5554':
        raise ValueError('Reserved emulator')
    command = ['adb', '-s', serial, 'shell', 'am', 'instrument', '-w']
    command += ['-e', 'a1' if args.a1 else 'boundaries', 'true']
    runner = ('com.jtn0123.audiowhisper.suggest.A1Instrumentation' if args.runner == 'a1'
              else 'com.jtn0123.audiowhisper.render.FixtureInstrumentation')
    command += ['com.jtn0123.audiowhisper.keyboard.test/' + runner]
    with STORES['fixture_session'](args.output / 'stores'):
        result = subprocess.run(command, capture_output=True, text=True, timeout=300)
        (args.output / 'instrumentation.log').write_text(result.stdout + result.stderr)
        if result.returncode or 'INSTRUMENTATION_CODE: -1' not in result.stdout:
            raise RuntimeError('Device comparison failed; retained instrumentation log')
        if args.a1:
            # The shipped dictionary's A1 replay, as A1SessionMatchTest writes it on the host to
            # app/build/stage2-a1/main_en-US/current/. The AOSP fixture's were daea23ac... / 658c8914...
            for key, value in {
                'decoder_sha256': 'a1c7aaa3bc37af33523b7e33be258c68b7ff1f7786e0a27a4b0b1cd79a887fd3',
                'pipeline_sha256': '26b57744452561b2309b2d24185e63a58222ea5f46acdacd53e29bd1d7d8ce51',
            }.items():
                if key + '=' + value not in result.stdout:
                    raise RuntimeError('A1 digest mismatch')
            report = {'a1': 'matched both accepted digests'}
        else:
            prefix = 'INSTRUMENTATION_RESULT: boundaries='
            report = json.loads(next(line[len(prefix):] for line in result.stdout.splitlines() if line.startswith(prefix)))
            if report['reader_reference_comparisons'] != 6_000:
                raise RuntimeError('Incomplete reader reference comparison')
            if report.get('reader_unit_roundtrips') != 65_536:
                raise RuntimeError('Incomplete UTF-16 transport comparison')
            if args.baseline and report['sha256'] != json.loads(args.baseline.read_text())['sha256']:
                raise RuntimeError('Boundary result digest mismatch')
        (args.output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Passed ' + ('A1 replay' if args.a1 else 'boundary replay and timing') + '; canonical restoration verified')


if __name__ == '__main__':
    main()
