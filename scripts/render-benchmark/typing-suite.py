"""Run each existing typing wrapper with its own verified canonical-store session.

python3 -I scripts/render-benchmark/typing-suite.py OUTPUT [--baseline BEFORE]
The baseline comparison belongs to each strict fixture. No successful result from another
run is reused. The input fixture accounts for the actual repeat schedule separately.
"""
import argparse
import json
from pathlib import Path
import runpy
import subprocess

stores = runpy.run_path(str(Path(__file__).with_name('store-session.py')))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--baseline', type=Path)
    parser.add_argument('--only', nargs='+')
    args = parser.parse_args()
    root = args.output
    root.mkdir(parents=True, exist_ok=True)
    results = []
    for name in ('regression', 'suggestions', 'editor', 'input', 'input-glide', 'modes', 'nlp'):
        if args.only and name not in args.only:
            continue
        output = root / name
        if name == 'regression':
            command = ['scripts/render-benchmark/regression.sh', str(output)]
        else:
            command = ['python3', '-I', 'scripts/render-benchmark/' + name + '.py', str(output)]
        if args.baseline and name in ('editor', 'input', 'input-glide', 'nlp'):
            filename = 'glide.json' if name == 'input-glide' else name + '.json'
            if name == 'nlp':
                command += ['--baseline']
            command += [str(args.baseline / name / filename)]
        print('Starting ' + name, flush=True)
        with stores['fixture_session'](root / (name + '-stores')):
            with (root / (name + '.log')).open('wb') as log:
                result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT)
        results.append({'suite': name, 'exit': result.returncode, 'canonical_restoration_verified': True})
        (root / 'typing-results.json').write_text(json.dumps(results, indent=2) + '\n')
        if result.returncode:
            raise RuntimeError(name + ' failed; inspect its private log')
        print('Passed ' + name + '; canonical hashes verified', flush=True)


if __name__ == '__main__':
    main()
