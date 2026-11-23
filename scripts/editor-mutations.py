"""Run five deliberate editor mutations through JVM differential tests, restoring every source.

Usage: python3 -I scripts/editor-mutations.py build/stage2-editor/mutations
Run only with no other Gradle process using this checkout.
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
SOURCE = ROOT / 'lib/rust/crates/editor/src'
# label: (file under SOURCE, old, new).
CASES = [
    ('invalid-finalize-batch', 'edit.rs',
     'vec![Op::simple(effect::BEGIN), Op::simple(effect::END)],',
     'vec![Op::simple(effect::BEGIN)],'),
    ('surrogate-offset', 'edit.rs',
     'let length = integer(fields, 0).max(0);',
     'let length = integer(fields, 0).max(0);\n'
     'let length = if length == 2 && c.before().last().is_some_and(|v| (0xdc00..=0xdfff).contains(v)) { length-1 } else { length };'),
    ('phantom-space', 'high.rs', 'let value = if phantom {', 'let value = if !phantom {'),
    ('batch-pair', 'edit.rs',
     'ops.extend([Op::simple(NEW_COMPOSING), Op::simple(effect::END)]);',
     'ops.extend([Op::simple(NEW_COMPOSING), Op::simple(effect::END)]);\n'
     'ops.retain(|op| !matches!(op.name,"begin"|"end"));'),
    ('reconciliation', 'content.rs', '&& self.composing() == composing', ''),
]


def check_edits():
    return [mutation_cases.Edit(label, SOURCE / file, old, new) for label, file, old, new in CASES]


def main():
    output = ROOT / sys.argv[1]
    output.mkdir(parents=True, exist_ok=True)
    edits = CASES
    results = []
    for label, file, old, new in edits:
        path = SOURCE / file
        original = path.read_bytes()
        source = original.decode()
        assert source.count(old) == 1, 'Mutation location changed: ' + label
        try:
            mutation_cases.write(path, source.replace(old, new))
            started = time.time()
            with (output / (label + '.log')).open('wb') as log:
                run = subprocess.run(['./gradlew', '--stacktrace', ':app:testDebugUnitTest',
                                      '--tests', '*EditorPlanMatchTest', '--tests', '*EditorFlowMatchTest'],
                                     cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
            failures = []
            for report in (ROOT / 'app/build/test-results/testDebugUnitTest').glob('TEST-*Editor*MatchTest.xml'):
                if report.stat().st_mtime < started:
                    continue
                suite = ET.parse(report).getroot()
                failures += [f.attrib.get('type', '') for f in suite.iter('failure')]
            caught = run.returncode != 0 and any('AssertionFailedError' in f for f in failures)
            results.append({'mutation': label, 'gradle_exit': run.returncode,
                            'assertion_failures': sum('AssertionFailedError' in f for f in failures),
                            'caught': caught, 'restored': True})
            (output / 'mutations.json').write_text(json.dumps(results, indent=2) + '\n')
            print(label + ': ' + ('caught' if caught else 'NOT CAUGHT'), flush=True)
            assert caught, 'Mutation did not fail a differential assertion: ' + label
        finally:
            mutation_cases.write(path, original)
    with (output / 'restored.log').open('wb') as log:
        run = subprocess.run(['./gradlew', '--stacktrace', ':app:testDebugUnitTest',
                              '--tests', '*Editor*Test'], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
    assert run.returncode == 0, 'Restored tests failed'
    print('All mutations restored; editor tests pass', flush=True)


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
