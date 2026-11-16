"""Break each auto-correction rule, require a real assertion failure, restore exact bytes, rerun green.

python3 -I scripts/correction-mutations.py OUTPUT [CASE...]
Host only: cargo tests and Gradle JVM tests, no device. Compilation errors do not count. The
rules and their measurements are in docs/autocorrect-results.md and docs/decisions.md.
"""
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
CRATES = 'lib/rust/crates/'
# name: (file, before, after, how to test). A test is ('cargo', package, test target or None,
# filter) or ('gradle', module, test class pattern).
CASES = {
    # Auto-correction strength.
    'normal-is-not-todays-threshold': (
        CRATES + 'suggest/src/auto_correction.rs',
        'Strength::Normal => Some(MODEST_THRESHOLD),', 'Strength::Normal => Some(MODEST_LEVEL),',
        ('cargo', 'suggest', None, 'auto_correction')),
    'modest-as-normal': (
        CRATES + 'suggest/src/auto_correction.rs',
        'Strength::Modest => Some(MODEST_LEVEL),', 'Strength::Modest => Some(MODEST_THRESHOLD),',
        ('cargo', 'suggest', None, 'auto_correction')),
    'off-keeps-own-spellings': (
        CRATES + 'suggest/src/suggester.rs',
        '        if self.threshold.is_none() {\n            return Ok(None);\n        }\n', '',
        ('cargo', 'suggest', None, 'off_never_corrects')),
    'jni-ignores-strength': (
        CRATES + 'awnative/src/suggest/session.rs',
        'threshold: Strength::from_ordinal(strength).threshold(),', 'threshold: Strength::DEFAULT.threshold(),',
        ('gradle', ':app', '*LatinImeJniTest')),
    'kotlin-ignores-setting': (
        'app/src/main/kotlin/com/jtn0123/audiowhisper/suggest/LatinSuggester.kt',
        'maxCount, strength().ordinal,', 'maxCount, AutoCorrectStrength.NORMAL.ordinal,',
        ('gradle', ':app', '*LatinImeJniTest')),
    # Backspace undoes an auto-correction.
    'late-updates-count-as-new': (
        CRATES + 'editor/src/session.rs',
        'let late = self.mass == 0 && own.is_some_and(|at| at + 1 < self.queue.len());', 'let late = false;',
        ('gradle', ':app', '*AutoCorrectionUndoFlowTest')),
    'late-updates-end-spacing': (
        'app/src/main/kotlin/dev/patrickgold/florisboard/ime/editor/EditorInstance.kt',
        '    if (late) return\n', '',
        ('gradle', ':app', '*AutoCorrectionUndoFlowTest')),
    'backspace-skips-the-undo': (
        CRATES + 'input/src/delete.rs',
        'out.extend([i64::from(DELETE_UNDO_AUTO), 0]);', 'out.extend([i64::from(DELETE_NOTIFY), 0]);',
        ('cargo', 'input', None, 'undo')),
    'other-keys-keep-the-undo': (
        CRATES + 'input/src/routing.rs',
        '    if code != DELETE {\n        add(CLEAR_UNDO, 0);\n    }\n', '',
        ('cargo', 'input', None, 'undo')),
    # Apostrophes: the language model decides between a word and its spelling with one.
    'model-never-restores': (
        CRATES + 'suggest/src/apostrophes.rs',
        'Some(odds) if odds >= MARGIN => Verdict::Restore,', 'Some(odds) if odds >= f32::MAX => Verdict::Restore,',
        ('cargo', 'suggest', None, 'apostrophe')),
    'whitelist-always-wins': (
        CRATES + 'suggest/src/suggester.rs',
        '                Some((Verdict::Keep, pair)) if std::ptr::eq(pair, top) => return Ok(None),\n', '',
        ('cargo', 'suggest', None, 'apostrophe')),
    'model-asked-in-lower-case': (
        CRATES + 'suggest/src/suggester.rs',
        'self.dictionary.ln_odds(&pair.display, typed)?', 'self.dictionary.ln_odds(&pair.raw.word, &lowercase(typed))?',
        ('cargo', 'suggest', None, 'apostrophe')),
    'restores-mid-word': (
        CRATES + 'suggest/src/suggester.rs',
        'if is_typed_valid && is_correctable(state) {', 'if is_typed_valid {',
        ('cargo', 'suggest', None, 'apostrophe')),
    'session-has-no-odds': (
        CRATES + 'suggest/src/session.rs',
        'Ok(model.and_then(|model| model.ln_odds(self.previous, word, other)))', 'Ok(model.and(None))',
        ('cargo', 'suggest', 'apostrophes', '')),
    'typist-types-apostrophes': (
        CRATES + 'scoreboard/src/typing.rs',
        '.filter(|&unit| !APOSTROPHES.contains(&unit))', '.filter(|&unit| unit != 0 || APOSTROPHES.is_empty())',
        ('cargo', 'scoreboard', None, 'apostrophes')),
}



WORKSPACE = ROOT / 'lib/rust'
# A failed test's panic: where it was raised, relative to the workspace.
PANIC = re.compile(r"panicked at ([^:\s]+):(\d+):\d+:")


def run(output, name, command, cwd=ROOT):
    with (output / (name + '.log')).open('wb') as log:
        result = subprocess.run(command, cwd=cwd, stdout=log, stderr=subprocess.STDOUT)
    (output / (name + '.exit')).write_text(str(result.returncode) + '\n')
    return result.returncode


def assertion_panics(text):
    """How many of the panics in a cargo test log were raised by an assert macro's line."""
    count = 0
    for path, line in PANIC.findall(text):
        source = WORKSPACE / path
        if source.is_file():
            count += 'assert' in source.read_text().splitlines()[int(line) - 1]
    return count


def check(output, name, test):
    """The test's exit status and how many assertions failed."""
    if test[0] == 'cargo':
        _, package, target, test_filter = test
        # From the workspace, so its .cargo/config.toml (target dir under build/) applies.
        command = ['cargo', 'test', '-p', package]
        command += ['--test', target] if target else ['--lib']
        status = run(output, name, command + ['--', test_filter], WORKSPACE)
        text = (output / (name + '.log')).read_text(errors='replace')
        compiled = 'error[E' not in text and 'error: could not compile' not in text
        return status, assertion_panics(text) if compiled else 0
    _, module, pattern = test
    reports = ROOT / module.strip(':').replace(':', '/') / 'build/test-results/testDebugUnitTest'
    if reports.exists():
        shutil.rmtree(reports)
    status = run(output, name, ['./gradlew', '--stacktrace', module + ':testDebugUnitTest', '--tests', pattern])
    assertions = 0
    for report in reports.glob('TEST-*.xml'):
        for failure in ET.parse(report).iter('failure'):
            kind = failure.get('type', '')
            assert 'AssertionFailedError' in kind or 'AssertionError' in kind, name + ': not an assertion failure'
            assertions += 1
        (output / (name + '-' + report.name)).write_bytes(report.read_bytes())
    return status, assertions


def main():
    output = Path(sys.argv[1]).resolve()
    output.mkdir(parents=True, exist_ok=True)
    names = sys.argv[2:] or list(CASES)
    results = []
    for name in names:
        relative, before, after, test = CASES[name]
        path = ROOT / relative
        original = path.read_bytes()
        source = original.decode()
        assert source.count(before) == 1, name + ': the mutation target must be unique'
        try:
            path.write_text(source.replace(before, after))
            status, assertions = check(output, name, test)
            assert status != 0 and assertions > 0, name + ': the mutation must fail by assertion, not compilation'
        finally:
            path.write_bytes(original)
        assert path.read_bytes() == original, name + ': byte restoration failed'
        restored, failures = check(output, name + '-restored', test)
        assert restored == 0 and failures == 0, name + ': the restored source must pass'
        results.append({'mutation': name, 'exit': status, 'assertion_failures': assertions,
                        'restored_exit': restored, 'restored_sha256': hashlib.sha256(original).hexdigest()})
        print(json.dumps(results[-1]), flush=True)
    (output / 'mutations.json').write_text(json.dumps(results, indent=2) + '\n')


if __name__ == '__main__':
    main()
