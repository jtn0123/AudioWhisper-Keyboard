"""Six deliberate NLP faults, actual JVM assertion failures, exact restoration. python3 -I.

Run with no other Gradle operation in this checkout. Never count a compilation failure as caught.
"""
import json
from pathlib import Path
import subprocess
import sys
import time
import xml.etree.ElementTree as ET

sys.dont_write_bytecode = True  # importing mutation_cases would otherwise leave a __pycache__ in scripts/
sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutation_cases

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'lib/rust/crates/nlp/src'
# label: (file under SOURCE, old, new).
CASES = [
    ('accept-stale-generation', 'session.rs', 'self.pending && generation == self.generation\n',
     'self.pending && generation <= self.generation\n'),
    ('wrong-dedup', 'session.rs', 'v.kind == value.kind && v.text == value.text',
     'v.kind == value.kind && v.text != value.text'),
    ('password-suggestions', 'policy.rs', '!password && !(raw & 15 == 1 && raw & 0x80000 != 0)',
     '(password || raw >= 0) && !(raw & 15 == 1 && raw & 0x80000 != 0)'),
    ('wrong-auto-expand', 'session.rs', 'self.visible().is_empty() && self.inline.is_empty() || self.context.selection()',
     'self.visible().is_empty() || self.inline.is_empty() || self.context.selection()'),
    # The Smartbar's words hold: typing must carry it, and an empty refill must end it.
    ('no-hold-across-keystroke', 'session.rs', 'self.carry = typing && holding;', 'self.carry = false;'),
    ('hold-outlives-empty-refill', 'session.rs', '|| self.carry || self.pending && self.held',
     '|| self.carry || self.held'),
]


def check_edits():
    return [mutation_cases.Edit(label, SOURCE / file, old, new) for label, file, old, new in CASES]


def main():
    output = ROOT / sys.argv[1]
    output.mkdir(parents=True, exist_ok=True)
    mutations = CASES
    if len(sys.argv) > 2:
        mutations = [edit for edit in mutations if edit[0] == sys.argv[2]]
        assert mutations, 'Unknown mutation selector'
    results = []
    for label, file, old, new in mutations:
        path = SOURCE / file
        original = path.read_bytes()
        assert original.decode().count(old) == 1, 'Mutation location changed: ' + label
        try:
            path.write_text(original.decode().replace(old, new))
            started = time.time()
            with (output / (label + '.log')).open('wb') as log:
                run = subprocess.run(['./gradlew', '--stacktrace', ':lib:rust:testDebugUnitTest', '--tests', '*NlpMatchTest'],
                    cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
            failures = []
            for report in (ROOT / 'lib/rust/build/test-results/testDebugUnitTest').glob('TEST-*Nlp*.xml'):
                if report.stat().st_mtime >= started:
                    failures += [f.attrib.get('type', '') for f in ET.parse(report).getroot().iter('failure')]
            caught = run.returncode != 0 and any('AssertionFailedError' in f for f in failures)
            results.append({'mutation': label, 'gradle_exit': run.returncode,
                'assertion_failures': sum('AssertionFailedError' in f for f in failures), 'caught': caught, 'restored': True})
            (output / 'mutations.json').write_text(json.dumps(results, indent=2) + '\n')
            print(label + ': ' + ('caught' if caught else 'NOT CAUGHT'), flush=True)
            assert caught, 'Mutation failed to trip an actual match assertion: ' + label
        finally:
            path.write_bytes(original)
    with (output / 'restored.log').open('wb') as log:
        run = subprocess.run(['./gradlew', '--stacktrace', ':lib:rust:testDebugUnitTest', '--tests', '*NlpMatchTest',
            ':app:testDebugUnitTest', '--tests', '*Nlp*MatchTest'], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
    assert run.returncode == 0, 'Restored match tests failed'
    print(f'All {len(mutations)} selected mutations restored; reference comparisons pass', flush=True)


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
