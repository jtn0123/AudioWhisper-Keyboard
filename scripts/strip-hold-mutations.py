"""Faults in the strip's hold across a refill (StripHold in render/ImeRootReads.kt, E56): each must
compile and then fail StripHoldTest; the source bytes are always restored. python3 -I.

    python3 -I scripts/strip-hold-mutations.py build/strip-hold/mutations [case ...]

The faults are the ways the hold goes wrong without looking wrong: words kept when Rust has let the
row go, kept past the deadline or for ever, a deadline that a second key pushes back, a wake asked
for once and never again, one never withdrawn, and an empty row "kept". ImeRootView's half (the
wake's Runnable and the read it replaces) needs a View and is checked on the device by the CPU
trials' strip counts. A compilation failure never counts as caught. Run with no other Gradle
operation in this checkout.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True  # importing mutation_cases would otherwise leave a __pycache__ in scripts/
sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutation_cases

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/render/ImeRootReads.kt'
TEST = 'com.jtn0123.audiowhisper.render.StripHoldTest'
RESULT = ROOT / f'app/build/test-results/testDebugUnitTest/TEST-{TEST}.xml'
KEEP = 'return drawn.also { wake(left) }'
# Each case: name, why, [(old, new)] in ImeRootReads.kt, each old held exactly once.
CASES = [
    ('kept-when-not-due', 'words kept after Rust lets the row go',
     [('candidates.isEmpty() && due && drawn.isNotEmpty()', 'candidates.isEmpty() && drawn.isNotEmpty()')]),
    ('kept-for-ever', 'the deadline never ends the hold', [(f'if (left > 0) {KEEP}', KEEP)]),
    ('kept-at-the-deadline', 'the hold lasts one millisecond too long', [('if (left > 0)', 'if (left >= 0)')]),
    ('deadline-pushed-back', 'each rebuild in the hold starts it again', [('if (since < 0) since = now', 'since = now')]),
    ('woken-once', 'the wake is asked for at the start only, not again at what is left',
     [(KEEP, 'return drawn.also { if (left == STRIP_HOLD_MS) wake(left) }')]),
    ('never-withdrawn', 'the wake stays asked for when the words come', [('        if (since >= 0) cancel()\n', '')]),
    ('nothing-kept', 'an empty strip is "kept" and wakes a rebuild',
     [('candidates.isEmpty() && due && drawn.isNotEmpty()', 'candidates.isEmpty() && due')]),
    ('drawn-forgotten', 'the words drawn are not remembered, so nothing is ever kept',
     [('        drawn = candidates\n        return candidates\n', '        return candidates\n')]),
]


def check_edits():
    return [mutation_cases.Edit(name, SOURCE, old, new) for name, _, edits in CASES for old, new in edits]


def gradle(log_path):
    """Whether StripHoldTest compiled and ran, and how many of its tests failed."""
    RESULT.unlink(missing_ok=True)
    with log_path.open('wb') as log:
        subprocess.run(['./gradlew', '--console=plain', ':app:testDebugUnitTest', '--tests', TEST],
                       cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=dict(os.environ))
    if not RESULT.exists():
        return False, 0
    counts = re.search(r'tests="(\d+)" skipped="\d+" failures="(\d+)" errors="(\d+)"', RESULT.read_text())
    return int(counts[1]) > 0, int(counts[2]) + int(counts[3])


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('output', type=Path)
    parser.add_argument('cases', nargs='*', help='case names to run (default: all)')
    args = parser.parse_args()
    unknown = set(args.cases) - {case[0] for case in CASES}
    assert not unknown, f'unknown cases: {sorted(unknown)}'
    cases = [case for case in CASES if not args.cases or case[0] in args.cases]
    output = ROOT / args.output
    output.mkdir(parents=True, exist_ok=False)
    original = SOURCE.read_bytes()
    ran, failed = gradle(output / 'baseline.log')
    assert ran and not failed, 'the unmutated source must pass first'
    results = []
    try:
        for name, why, edits in cases:
            source = original.decode()
            for old, new in edits:
                assert source.count(old) == 1, f'mutation location changed: {name}: {old!r}'
                source = source.replace(old, new)
            SOURCE.write_text(source)
            try:
                ran, failed = gradle(output / (name + '.log'))
            finally:
                SOURCE.write_bytes(original)
            assert ran, f'{name} did not compile or run; that never counts as caught'
            assert failed, f'{name} survived: StripHoldTest passed'
            results.append({'mutation': name, 'fault': why, 'failed_tests': failed})
            print(f'{name}: caught by {failed} test(s)', flush=True)
    finally:
        SOURCE.write_bytes(original)
    ran, failed = gradle(output / 'restored.log')
    assert ran and not failed, 'restored source fails'
    report = {'mutations': results, 'restored_failures': failed,
              'source_sha256': hashlib.sha256(SOURCE.read_bytes()).hexdigest()}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'All {len(results)} mutations caught; restored source passes', flush=True)


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
