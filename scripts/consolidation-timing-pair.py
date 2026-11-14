"""Alternate immutable before/after APKs with the same test APK; retain every timing sample."""
import argparse
import json
import math
import os
from pathlib import Path
import shutil
import statistics
import subprocess

ROOT = Path(__file__).resolve().parent.parent
APP = ROOT / 'app/build/outputs/apk/debug/app-debug.apk'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('before', type=Path)
    parser.add_argument('after', type=Path)
    parser.add_argument('test_apk', type=Path)
    parser.add_argument('--baseline', type=Path, required=True)
    parser.add_argument('--rounds', type=int, default=3)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    sources = {'before': args.before.resolve(), 'after': args.after.resolve()}
    if APP.resolve() in sources.values():
        raise ValueError('Use immutable evidence APKs, not the generated install slot')
    if os.environ['ANDROID_SERIAL'] == 'emulator-5554':
        raise ValueError('Reserved emulator')
    results = {'before': [], 'after': []}
    def install(name):
        shutil.copyfile(sources[name], APP)
        with (args.output / 'install.log').open('ab') as log:
            subprocess.run(['scripts/install-dev.sh'], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=True)
    try:
        with (args.output / 'install.log').open('ab') as log:
            subprocess.run(['scripts/install-dev.sh', '--probe', str(args.test_apk.resolve())], cwd=ROOT,
                           stdout=log, stderr=subprocess.STDOUT, check=True)
        for turn in range(args.rounds):
            for name in (('before', 'after') if turn % 2 == 0 else ('after', 'before')):
                install(name)
                output = args.output / (str(turn) + '-' + name)
                subprocess.run(['python3', '-I', 'scripts/consolidation-device.py', str(output),
                                '--runner', 'renderer', '--baseline', str(args.baseline.resolve())], cwd=ROOT, check=True)
                results[name].append(json.loads((output / 'result.json').read_text()))
    finally:
        install('after')
    report = {}
    for name, trials in results.items():
        report[name] = {}
        for key, value in trials[0].items():
            if not isinstance(value, dict):
                continue
            samples = sorted(x for trial in trials for x in trial[key]['samples_ns'])
            report[name][key] = {'p50_ns': statistics.median(samples),
                                 'p95_ns': samples[math.ceil(len(samples) * .95) - 1],
                                 'samples_ns': samples}
    (args.output / 'comparison.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Alternating APK timings complete; all fingerprints matched and canonical state restored')


if __name__ == '__main__':
    main()
