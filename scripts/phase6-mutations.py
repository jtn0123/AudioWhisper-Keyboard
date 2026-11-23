"""Faults in code whose frozen Kotlin reference is retired (docs/phase6-kotlin-references.md), for the
JVM tests that compared against it and now check golden digests instead. python3 -I.

    python3 -I scripts/phase6-mutations.py OUTPUT --family dictation [case ...]

Each case changes one place in Rust or Kotlin, runs the JVM test classes it names in one Gradle
call, and counts as caught only by an AssertionFailedError in a fresh test report: a compilation
failure never counts. The source is restored byte for byte after each case, and the restored
source must pass every target. Run it at the commit before a family's references go and at the one
after: the goldens must catch every case the reference caught.

Run with no other Cargo or Gradle operation in this checkout: the source is mutated while it runs.
"""
import argparse
import hashlib
import importlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent
sys.dont_write_bytecode = True  # importing the cases and mutation_cases would otherwise leave a __pycache__ in scripts/
sys.path.insert(0, str(ROOT / 'scripts'))
import mutation_cases


def families():
    """Every family in scripts/phase6_mutation_cases, by name."""
    folder = ROOT / 'scripts/phase6_mutation_cases'
    return sorted(path.stem for path in folder.glob('*.py') if path.stem != '__init__')


def check_edits():
    return [edit._replace(case=family + ':' + edit.case) for family in families()
            for edit in mutation_cases.rows(importlib.import_module('phase6_mutation_cases.' + family).CASES)]


def env():
    e = dict(os.environ)
    e.setdefault('CARGO_BUILD_JOBS', '6')
    if 'ANDROID_HOME' not in e:
        for line in (ROOT / 'local.properties').read_text().splitlines():
            if line.startswith('sdk.dir='):
                e['ANDROID_HOME'] = line.split('=', 1)[1]
    return e


def jvm(output, name, module, patterns, task='testDebugUnitTest'):
    """Runs the test classes matching patterns in module's JVM test task (an Android module's
    testDebugUnitTest unless the family names its own TASK, as a plain JVM module's `test`);
    returns the exit and fresh assertion failures."""
    started = time.time()
    command = ['./gradlew', '--stacktrace', module + ':' + task]
    for pattern in patterns:
        command += ['--tests', pattern]
    with (output / (name + '.log')).open('wb') as log:
        result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=env())
    reports = ROOT / module.strip(':').replace(':', '/') / 'build/test-results' / task
    failures = 0
    for report in reports.glob('TEST-*.xml'):
        if report.stat().st_mtime >= started:
            failures += sum('AssertionFailedError' in f.attrib.get('type', '')
                            for f in ET.parse(report).getroot().iter('failure'))
    return result.returncode, failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--family', required=True, help='a module in scripts/phase6_mutation_cases')
    parser.add_argument('cases', nargs='*', help='case names to run (default: all of the family)')
    args = parser.parse_args()
    family = importlib.import_module('phase6_mutation_cases.' + args.family)
    every = family.CASES
    unknown = set(args.cases) - {case[0] for case in every}
    assert not unknown, f'unknown cases: {sorted(unknown)}'
    cases = [case for case in every if not args.cases or case[0] in args.cases]
    task = getattr(family, 'TASK', 'testDebugUnitTest')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    paths = {ROOT / relative for _, relative, _, _, _ in cases}
    originals = {path: path.read_bytes() for path in paths}
    for name, relative, old, _, _ in cases:
        assert originals[ROOT / relative].decode().count(old) == 1, 'mutation location changed: ' + name
    results = []
    try:
        for name, relative, old, new, target in cases:
            path = ROOT / relative
            mutation_cases.write(path, originals[path].decode().replace(old, new))
            try:
                status, failures = jvm(output, name, family.MODULE, family.TARGETS[target], task)
            finally:
                mutation_cases.write(path, originals[path])
            caught = status != 0 and failures > 0
            results.append({'mutation': name, 'file': relative, 'target': target, 'exit': status,
                            'assertion_failures': failures, 'caught': caught})
            (output / 'result.json').write_text(json.dumps({'mutations': results}, indent=2) + '\n')
            print(name + (': caught by ' + target if caught else ': NOT CAUGHT'), flush=True)
    finally:
        for path, original in originals.items():
            mutation_cases.write(path, original)
    patterns = sorted({p for _, _, _, _, target in cases for p in family.TARGETS[target]})
    status, failures = jvm(output, 'restored', family.MODULE, patterns, task)
    missed = [r['mutation'] for r in results if not r['caught']]
    report = {'mutations': results, 'missed': missed, 'restored': {'exit': status, 'assertion_failures': failures},
              'source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    assert status == 0 and failures == 0, 'restored source fails'
    print(f'{len(results) - len(missed)} of {len(results)} mutations caught; restored source passes', flush=True)


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
