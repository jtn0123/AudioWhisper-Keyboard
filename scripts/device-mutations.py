"""Faults the device comparisons must catch (docs/phase6-kotlin-references.md, owed item E54), run on
an emulator through scripts/device-goldens.py's steps. python3 -I.

    python3 -I scripts/device-mutations.py OUTPUT --family boundary [case ...]

Each case changes one place in production code, builds and installs the family's fixture APKs and
runs its instrumentation. It counts as caught only when the run fails at the comparison: before a
family's references go, the reference's own check (or, for a1, the accepted digests), after, the
golden tape. A mutant that does not build, or fails anywhere else, is not caught. The source is
restored byte for byte after each case, and the restored build must pass. Run it at a family's
generator commit and at its deletion commit: the goldens must catch every case the references did.
Dictation's device cases are scripts/dictation-mutations.py's `device` mode.

A family whose reference a JVM test compares too names that test in JVM_TESTS: each case must
also fail it by assertion, and the restored source pass it.

Run with no other Cargo or Gradle operation in this checkout: the source is mutated while it runs.
"""
import argparse
import json
from pathlib import Path
import re
import sys
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent
sys.dont_write_bytecode = True  # importing mutation_cases would otherwise leave a __pycache__ in scripts/
sys.path.insert(0, str(ROOT / 'scripts'))
import mutation_cases

DEVICE = mutation_cases.load(ROOT / 'scripts/device-goldens.py')

# By family: (name, file, before, after).
CASES = {
    'gif': [
        ('typing-ignored', 'lib/rust/crates/gif/src/session.rs',
         'ops::TYPE => self.query = search_text::type_text(&self.query, fields.take()?),',
         'ops::TYPE => { let _typed = fields.take()?; }'),
    ],
    'extension': [
        ('maintainer-email-and-url-swapped', 'app/src/main/kotlin/dev/patrickgold/florisboard/lib/ext/ExtensionMaintainer.kt',
         'part(EMAIL, EMAIL_END), part(URL, URL_END)', 'part(URL, URL_END), part(EMAIL, EMAIL_END)'),
    ],
    'boundary': [
        ('resume-at-the-first-anchor', 'lib/rust/crates/reader/src/page_text.rs',
         'blocks.windows(length).rposition(|it| it == anchor)', 'blocks.windows(length).position(|it| it == anchor)'),
    ],
    'a1': [
        # The native session's own (latin-mutations.py has it too): the reference engine, the AOSP
        # glue, never runs it, so the reference comparison on the JVM can see it as well.
        ('session-weight-given-not-chosen', 'lib/rust/crates/suggest/src/session.rs',
         'self.output.weight = -1.0;\n        let options', 'self.output.weight = 0.5;\n        let options'),
    ],
    'tts': [
        ('fewer-flow-steps', 'lib/rust/crates/sherpa-ffi/src/tts.rs', 'num_steps: 5,', 'num_steps: 4,'),
    ],
}

# The shipped dictionary's A1 replay as the host's A1SessionMatchTest writes it (scripts/consolidation-device.py).
ACCEPTED = {
    'decoder_sha256': 'a1c7aaa3bc37af33523b7e33be258c68b7ff1f7786e0a27a4b0b1cd79a887fd3',
    'pipeline_sha256': '26b57744452561b2309b2d24185e63a58222ea5f46acdacd53e29bd1d7d8ce51',
}

# JVM tests comparing the same reference, by family (app's unit tests).
JVM_TESTS = {
    'extension': ('dev.patrickgold.florisboard.lib.ext.ExtensionMaintainerMatchTest',),
    'a1': ('com.jtn0123.audiowhisper.suggest.A1SessionMatchTest',),
}
JVM_RESULTS = ROOT / 'app/build/test-results/testDebugUnitTest'

# A failure at the comparison, by family: the reference's check, or the golden tape (named by its prefix).
FAILED_AT_COMPARISON = {
    'gif': r'failure=(org\.junit\.ComparisonFailure$|java\.lang\.AssertionError: gif-search-typing: )',
    'extension': r'failure=java\.lang\.(IllegalStateException: Extension maintainer (parse|format) mismatch'
                 r'|AssertionError: extension-maintainer-)',
    'boundary': r'failure=java\.lang\.(IllegalStateException: reader (blocks|find|resume) sample \d+'
                r'|AssertionError: boundary-)',
    'a1': r'failure=java\.lang\.AssertionError: a1-',
    'tts': r'failure_kind=parity',
}


def check_edits():
    return mutation_cases.rows(case for cases in CASES.values() for case in cases)


def accepted(log):
    """Whether an a1 run reported the accepted digests."""
    return all(f'{key}={value}' in log for key, value in ACCEPTED.items())


def verdict(family, passed, log):
    """How a mutant's run was caught ('comparison', or 'digest' for a1 before its goldens), or None."""
    if not passed:
        return 'comparison' if re.search(FAILED_AT_COMPARISON[family], log, re.M) else None
    return 'digest' if family == 'a1' and not accepted(log) else None


def run(output, family_name, name):
    """Builds, installs and runs the family; returns whether the build passed, the run passed, and its log."""
    family = DEVICE.FAMILIES[family_name]
    try:
        DEVICE.build(output, family, name)
    except AssertionError:
        return False, False, ''
    passed, log = DEVICE.instrument(output, family, name + '-run')
    return True, passed, log


def assertion_failures(test):
    """How many of a JVM test class's cases failed by assertion in its last run (none without a report)."""
    report = JVM_RESULTS / f'TEST-{test}.xml'
    if not report.exists():
        return 0
    return sum(1 for failure in ET.parse(report).getroot().iter('failure') if 'Assertion' in failure.get('type', ''))


def jvm(output, family_name, name):
    """Runs the family's JVM tests: 'passed', 'caught' (an assertion failed in them), 'failed
    elsewhere', or None when the family has none."""
    tests = JVM_TESTS.get(family_name)
    if not tests:
        return None
    for test in tests:
        (JVM_RESULTS / f'TEST-{test}.xml').unlink(missing_ok=True)
    command = ['./gradlew', '--stacktrace', ':app:testDebugUnitTest', '--rerun']
    for test in tests:
        command += ['--tests', test]
    status, _ = DEVICE.logged(output, name + '-jvm', command)
    if status == 0:
        return 'passed'
    return 'caught' if any(assertion_failures(test) for test in tests) else 'failed elsewhere'


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('output', type=Path)
    parser.add_argument('--family', required=True, choices=sorted(CASES))
    parser.add_argument('cases', nargs='*', help='case names to run (default: all of the family)')
    args = parser.parse_args()
    every = CASES[args.family]
    unknown = set(args.cases) - {case[0] for case in every}
    assert not unknown, f'unknown cases: {sorted(unknown)}'
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    results = []
    for name, relative, before, after in every:
        if args.cases and name not in args.cases:
            continue
        path = ROOT / relative
        original = path.read_bytes()
        source = original.decode()
        assert source.count(before) == 1, name + ': the search text is not there exactly once'
        try:
            mutation_cases.write(path, source.replace(before, after))
            mutant_built, passed, log = run(output, args.family, name)
            caught = verdict(args.family, passed, log) if mutant_built else None
            on_jvm = jvm(output, args.family, name)
        finally:
            mutation_cases.write(path, original)
        assert path.read_bytes() == original, name + ': not restored'
        built, passed, log = run(output, args.family, name + '-restored')
        restored = built and passed and (args.family != 'a1' or accepted(log))
        assert restored, name + ': the restored source must build and pass'
        assert jvm(output, args.family, name + '-restored') in (None, 'passed'), name + ': the restored JVM tests must pass'
        results.append({'case': name, 'mutant_built': mutant_built, 'caught': caught, 'jvm': on_jvm})
        shown = '' if on_jvm is None else f'; on the JVM, {on_jvm}'
        print(f'{args.family} {name}: ' + (f'caught at the {caught}' if caught else 'NOT caught') + shown + '; restored passes',
              flush=True)
    (output / 'mutations.json').write_text(json.dumps(results, indent=2) + '\n')
    caught = sum(1 for result in results if result['caught'] and result['jvm'] in (None, 'caught'))
    print(f'{args.family}: {caught} of {len(results)} caught' + (', on the device and the JVM' if args.family in JVM_TESTS else ''))


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
