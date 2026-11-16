"""Roomy Smartbar and word-shrink faults: each must trip fresh Rust test panics and/or fresh JVM
assertion failures in every checker it names; source bytes are always restored. python3 -I.

    python3 -I scripts/smartbar-mutations.py build/smartbar-roomy/mutations [case ...]

Run with no other Gradle or Cargo operation in this checkout. A compilation failure never counts
as caught: Rust needs a "panicked at" line, the JVM an AssertionFailedError in a fresh report.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent
RENDER = ROOT / 'lib/rust/crates/render/src'
BAR = RENDER / 'smartbar.rs'
RUST, JVM = 'rust:smartbar_roomy', 'jvm:SmartbarMatchTest'
CASES = [
    ('roomy-without-words', BAR, '            && self.words\n', '', [RUST, JVM]),
    ('roomy-while-dictating', BAR, '            && !self.dictating\n', '', [RUST, JVM]),
    ('roomy-hides-the-reading-stop', BAR, '            && !(self.reading && self.speaker)\n', '', [RUST, JVM]),
    ('roomy-over-inline-autofill', BAR, '            && !self.inline_visible()\n', '', [RUST, JVM]),
    ('pinned-kept-with-words', BAR, 'if c.layout >= 2 && !c.roomy() {', 'if c.layout >= 2 {', [RUST, JVM]),
    ('extended-row-dropped', BAR, '    if c.extra() {\n        let top', '    if c.extra() && !c.roomy() {\n        let top',
     [RUST, JVM]),
    ('words-fact-not-decoded', BAR, 'words: a.get(14).is_some_and(|v| *v != 0.0),', 'words: a.get(15).is_some_and(|v| *v != 0.0),',
     [RUST, JVM]),
    ('no-shrink-floor', BAR, 'fit.max(MIN_TEXT_SCALE).min(scale)', 'fit.max(0.5).min(scale)', [RUST, JVM]),
    ('shrink-not-whole-percent', BAR, '(scale * available.max(0.0) / measured * 100.0).floor() / 100.0',
     'scale * available.max(0.0) / measured', [RUST, JVM]),
    ('raster-never-shrinks', RENDER / 'smartbar_raster.rs', '* crate::smartbar::fit_text(r.width(), |scale| {',
     '* crate::smartbar::fit_text(f32::INFINITY, |scale| {', [RUST]),
]


def env():
    e = dict(os.environ)
    e.setdefault('CARGO_BUILD_JOBS', '6')
    return e


def rust(output, name, target):
    with (output / (name + '.' + target + '.log')).open('wb') as log:
        result = subprocess.run(['cargo', 'test', '-q', '-p', 'render', '--test', target],
                                cwd=ROOT / 'lib/rust', stdout=log, stderr=subprocess.STDOUT, env=env())
    text = (output / (name + '.' + target + '.log')).read_text(errors='replace')
    return result.returncode, sum('panicked at' in line for line in text.splitlines())


def jvm(output, name, test):
    started = time.time_ns()
    with (output / (name + '.' + test + '.log')).open('wb') as log:
        result = subprocess.run(['./gradlew', '--max-workers=6', ':lib:rust:testDebugUnitTest', '--tests', '*' + test],
                                cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=env())
    failures = 0
    for path in (ROOT / 'lib/rust/build/test-results/testDebugUnitTest').glob('TEST-*' + test + '.xml'):
        if path.stat().st_mtime_ns >= started:
            failures += sum('AssertionFailedError' in f.attrib.get('type', '')
                            for f in ET.parse(path).getroot().iter('failure'))
    return result.returncode, failures


def check(output, name, checker):
    kind, target = checker.split(':')
    return (rust if kind == 'rust' else jvm)(output, name, target)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('cases', nargs='*', help='case names to run (default: all)')
    args = parser.parse_args()
    unknown = set(args.cases) - {case[0] for case in CASES}
    assert not unknown, f'unknown cases: {sorted(unknown)}'
    cases = [case for case in CASES if not args.cases or case[0] in args.cases]
    output = ROOT / args.output
    output.mkdir(parents=True, exist_ok=False)
    originals = {path: path.read_bytes() for _, path, _, _, _ in cases}
    results = []
    try:
        for name, path, old, new, checkers in cases:
            source = originals[path].decode()
            assert source.count(old) == 1, 'mutation location changed: ' + name
            path.write_text(source.replace(old, new))
            try:
                caught = {}
                for checker in checkers:
                    status, failures = check(output, name, checker)
                    assert status != 0 and failures, f'{name} must fail fresh assertions in {checker}'
                    caught[checker] = failures
                results.append({'mutation': name, 'caught': caught})
            finally:
                path.write_bytes(originals[path])
            print(name + ': caught by ' + ', '.join(f'{c} ({n})' for c, n in caught.items()), flush=True)
    finally:
        for path, original in originals.items():
            path.write_bytes(original)
    restored = {}
    for checker in sorted({c for case in cases for c in case[4]}):
        status, failures = check(output, 'restored', checker)
        assert status == 0 and not failures, 'restored source fails ' + checker
        restored[checker] = status
    report = {'mutations': results, 'restored_exit': restored,
              'source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in originals}}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'All {len(results)} mutations caught; restored source passes {sorted(restored)}', flush=True)


if __name__ == '__main__':
    main()
