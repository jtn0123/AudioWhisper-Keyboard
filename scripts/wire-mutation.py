"""Break crates/wire five ways; require the build to reject a mismatch, or both sides to follow.

python3 -I scripts/wire-mutation.py OUTPUT [CASE...]
The cases: two operations given one code, two given one name, an operation renamed with its Rust
users updated but not its Kotlin caller, the same rename with its Rust users left behind, and two
operations given new values with nothing else touched. The first four must fail to build. The last must
build and pass the tests of both families, with the generated Kotlin carrying the new values. Each
case is restored byte for byte and must pass again.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent
WIRE = 'lib/rust/crates/wire/src/'
GENERATED = 'lib/rust/build/generated/nativeBoundary/kotlin/com/jtn0123/audiowhisper/rust/'
CARGO_WIRE = (['cargo', 'check', '-p', 'wire'], 'lib/rust')
CARGO_DICTATION = (['cargo', 'check', '-p', 'dictation'], 'lib/rust')
KOTLIN = (['./gradlew', '--stacktrace', ':app:compileDebugKotlin'], '.')
FAMILY_TESTS = (['./gradlew', '--stacktrace', ':app:testDebugUnitTest',
                 '--tests', 'com.jtn0123.audiowhisper.reader.*',
                 '--tests', 'com.jtn0123.audiowhisper.dictation.*'], '.')


def replace_once(old, new):
    """A mutation replacing `old`, which must occur exactly once, with `new`."""
    def change(source):
        assert source.count(old) == 1, 'expected exactly one ' + repr(old)
        return source.replace(old, new)
    return change


# name: (edits, command, whether it must pass, lines its log must hold, generated Kotlin to find).
CASES = {
    'duplicate-code': (
        [(WIRE + 'playback.rs', replace_once('BEGINS = 1,', 'BEGINS = 0,'))],
        CARGO_WIRE, False, ['two constants of one group share a value'], []),
    'duplicate-name': (
        [(WIRE + 'playback.rs', replace_once('ENDS = 2,', 'BEGINS = 2,'))],
        CARGO_WIRE, False, ['the name `BEGINS` is defined multiple times'], []),
    # Rust follows the rename; the Kotlin caller still says PRESS.
    'renamed-kotlin-behind': (
        [(WIRE + 'dictation.rs', replace_once('PRESS = "press",', 'TAP = "press",')),
         ('lib/rust/crates/dictation/src/session.rs', replace_once('events::PRESS =>', 'events::TAP =>'))],
        KOTLIN, False, ["Unresolved reference 'PRESS'", 'DictationSession.kt'], []),
    # The same rename, with the Rust user left behind.
    'renamed-rust-behind': (
        [(WIRE + 'dictation.rs', replace_once('PRESS = "press",', 'TAP = "press",'))],
        CARGO_DICTATION, False, ['cannot find unit struct, unit variant or constant `PRESS` in module `events`'], []),
    # New values on the wire, nothing else touched: both sides follow, and both families still pass.
    'renumbered-followed': (
        [(WIRE + 'playback.rs', replace_once('HEARD = 3,', 'HEARD = 30,')),
         (WIRE + 'dictation.rs', replace_once('PRESS = "press",', 'PRESS = "tap",'))],
        FAMILY_TESTS, True, ['BUILD SUCCESSFUL'],
        [('PlaybackOp.kt', 'HEARD(30)'), ('DictationEvent.kt', 'PRESS("tap")')]),
}


def run(output, phase, command):
    arguments, directory = command
    with (output / (phase + '.log')).open('wb') as log:
        return subprocess.run(arguments, cwd=ROOT / directory, stdout=log,
                              stderr=subprocess.STDOUT).returncode


def mutate(output, name):
    edits, command, passes, expected, generated = CASES[name]
    originals = {relative: (ROOT / relative).read_bytes() for relative, _ in edits}
    try:
        for relative, change in edits:
            source = (ROOT / relative).read_text()
            mutated = change(source)
            assert mutated != source, name + ': the mutation must change ' + relative
            (ROOT / relative).write_text(mutated)
        mutant = run(output, name, command)
        log = (output / (name + '.log')).read_text()
        assert (mutant == 0) == passes, name + (': the build must pass' if passes else ': the build must fail')
        for line in expected:
            assert line in log, name + ': missing ' + line
        for file, text in generated:
            assert text in (ROOT / GENERATED / file).read_text(), name + ': ' + file + ' lacks ' + text
    finally:
        for relative, original in originals.items():
            (ROOT / relative).write_bytes(original)
    for relative, original in originals.items():
        assert (ROOT / relative).read_bytes() == original, name + ': byte restoration failed for ' + relative
    restored = run(output, name + '-restored', command)
    assert restored == 0, name + ': the restored source must pass'
    digest = hashlib.sha256(b''.join(originals[relative] for relative in sorted(originals)))
    return {'mutation': name, 'files': sorted(originals), 'command': ' '.join(command[0]),
            'mutant_exit': mutant, 'restored_exit': restored,
            'restored_source_sha256': digest.hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('cases', nargs='*', help='default: all of ' + ', '.join(CASES))
    args = parser.parse_args()
    names = args.cases or list(CASES)
    unknown = [name for name in names if name not in CASES]
    if unknown:
        parser.error('unknown case: ' + ', '.join(unknown))
    args.output.mkdir(parents=True, exist_ok=False)
    results = []
    for name in names:
        results.append(mutate(args.output, name))
        print(json.dumps(results[-1]), flush=True)
    (args.output / 'result.json').write_text(json.dumps(results, indent=2) + '\n')


if __name__ == '__main__':
    main()
