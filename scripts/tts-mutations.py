"""Require real assertion failures from production mutations, restore exact bytes, rerun green.

python3 -I scripts/tts-mutations.py OUTPUT input|tts
No device operations. Emulator acceptance is a separate required proof.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import xml.etree.ElementTree as ET

sys.dont_write_bytecode = True  # importing mutation_cases would otherwise leave a __pycache__ in scripts/
sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutation_cases

root = Path(__file__).resolve().parents[1]
CRATES = root / 'lib/rust/crates'
# By mode: (name, file under CRATES, before, after, JVM suite, or None for the reader's cargo test).
CASES = {
    'input': [
        ('repeat-plus-one-ms', 'input/src/lifecycle.rs',
         '    (delay as f32 * factor) as i64\n}\npub fn repeat_code',
         '    (delay as f32 * factor) as i64 + 1\n}\npub fn repeat_code', '*InputDispatchMatchTest'),
    ],
    'tts': [
        ('chunk-boundary', 'reader/src/page_text.rs',
         'current.len() + sentence.len() > max', 'current.len() + sentence.len() >= max', '*ReaderRulesMatchTest'),
        ('stop-before-free', 'reader/src/generator.rs',
         '        self.stopped.store(true, Ordering::Release);', '        // Deliberate mutation: forget stop before releasing the model.', None),
        ('password-screen-rule', 'reader/src/walk.rs',
         'if node.password || node.editable', 'if node.editable', '*ScreenTraversalMatchTest'),
    ],
}


def check_edits():
    return [mutation_cases.Edit(name, CRATES / relative, before, after)
            for cases in CASES.values() for name, relative, before, after, _ in cases]


def run(out, command, name):
    with (out / (name + '.log')).open('wb') as log:
        result = subprocess.run(command, cwd=root, stdout=log, stderr=subprocess.STDOUT)
    (out / (name + '.exit')).write_text(str(result.returncode) + '\n')
    return result.returncode


def check(out, name, suite):
    if suite:
        reports = root / 'app/build/test-results/testDebugUnitTest'
        for report in reports.glob('TEST-*' + suite.removeprefix('*') + '.xml'):
            report.unlink()
        status = run(out, ['./gradlew', ':app:testDebugUnitTest', '--tests', suite], name)
        failures = 0
        for path in reports.glob('TEST-*' + suite.removeprefix('*') + '.xml'):
            node = ET.parse(path).getroot()
            for failure in node.findall('.//failure'):
                assert failure.get('type') in ('org.opentest4j.AssertionFailedError', 'java.lang.AssertionError'), 'Not an assertion failure'
            failures += int(node.get('failures', '0'))
            (out / (name + '-' + path.name)).write_bytes(path.read_bytes())
        return status, failures
    status = run(out, ['cargo', 'test', '--locked', '--manifest-path', 'lib/rust/Cargo.toml',
                       '--target-dir', 'build/stage2-tts/cargo-standalone', '-p', 'reader',
                       'generator::tests::stop_precedes_free_and_callback_keeps_buffer_live'], name)
    text = (out / (name + '.log')).read_text()
    return status, int('test result: FAILED' in text and 'panicked at' in text)


def main():
    out = Path(sys.argv[1]).resolve(); out.mkdir(parents=True, exist_ok=True)
    mode = sys.argv[2]
    assert mode in CASES
    results = []
    for name, relative, before, after, suite in CASES[mode]:
        path = CRATES / relative
        original = path.read_bytes(); source = original.decode()
        assert source.count(before) == 1, name
        try:
            path.write_text(source.replace(before, after))
            status, failures = check(out, name, suite)
            assert status != 0 and failures > 0, name + ': mutation must fail by assertion, not compilation'
            results.append({'mutation': name, 'exit': status, 'assertion_failures': failures,
                            'restored_sha256': hashlib.sha256(original).hexdigest()})
        finally:
            path.write_bytes(original)
        assert path.read_bytes() == original
        status, failures = check(out, name + '-restored', suite)
        assert status == 0 and failures == 0, name + ': restored code must pass'
    (out / 'mutations.json').write_text(json.dumps(results, indent=2) + '\n')
    print(json.dumps(results))


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
