"""Break each swipe-accuracy change, require a real assertion failure, restore exact bytes, rerun green.

python3 -I scripts/glide-mutations.py OUTPUT [CASE...]
Host only: cargo tests and Gradle JVM tests, no device. Compilation errors do not count. The
changes and their measurements are in docs/swipe-accuracy.md.
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
    'ends-measured-from-first-key': (
        CRATES + 'glide/src/tuning.rs',
        'let d1 = distance(swipe.x(end), swipe.y(end), last.0, last.1) / spread;',
        'let d1 = distance(swipe.x(end), swipe.y(end), first.0, first.1) / spread;',
        ('cargo', 'glide', None, 'ends')),
    'dwell-favours-fast-stretches': (
        CRATES + 'glide/src/tuning.rs',
        '.powf(dwell)', '.powf(-dwell)',
        ('cargo', 'glide', None, 'dwell')),
    'reach-ignores-keys-off-the-swipe': (
        CRATES + 'glide/src/tuning.rs',
        'let off = beyond(farthest(path, swipe));', 'let off = 0.0f32;',
        ('cargo', 'glide', None, 'reach')),
    'curve-flat-above-common': (
        CRATES + 'suggest/src/swipe_words.rs',
        '        below: COST_PER_STEP,\n        above: COST_PER_STEP,\n',
        '        below: COST_PER_STEP,\n        above: 0.0,\n',
        ('cargo', 'suggest', None, 'swipe_words')),
    'boost-skips-first-doubling': (
        CRATES + 'suggest/src/swipe_words.rs',
        'let doublings = (uses / KNOWN_AFTER).ilog2() as i32 + 1;',
        'let doublings = (uses / KNOWN_AFTER).ilog2() as i32;',
        ('cargo', 'suggest', None, 'swipe_words')),
    'boost-ignores-spelling': (
        CRATES + 'suggest/src/swipe_words.rs',
        'if learned.spelling_of(word) == Some(word) {',
        'if learned.spelling_of(word).is_some() {',
        ('cargo', 'suggest', None, 'swipe_words')),
    'renderer-times-unasked': (
        CRATES + 'render/src/glide_touch.rs',
        'if c.glide_times {', 'if !c.glide_times {',
        ('cargo', 'render', 'glide_times', '')),
    'jni-scores-as-original': (
        CRATES + 'awnative/src/glide.rs',
        'score(env, query, to, &Tuning::DEFAULT)', 'score(env, query, to, &Tuning::ORIGINAL)',
        ('gradle', ':lib:rust', '*GlideIndexTest')),
    'index-drops-times': (
        'app/src/main/kotlin/dev/patrickgold/florisboard/ime/text/gestures/SwipeIndex.kt',
        'index.query(swipe.xs(), swipe.ys(), swipe.times(), glideKeys(keys))',
        'index.query(swipe.xs(), swipe.ys(), FloatArray(0), glideKeys(keys))',
        ('gradle', ':app', '*SwipeIndexTest')),
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
