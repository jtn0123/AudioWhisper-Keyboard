"""Break the native boundary six ways; require the APK gate to reject each, then pass restored.

python3 -I scripts/native-declarations-mutation.py OUTPUT [CASE...]
The cases: a production Kotlin declaration with no export, an export no Kotlin declares, an export
only tests declare left in the phone's library, a hand-written Rust entry point reading a Long as
a jint, a generated declaration whose type changed on one side only, and a renamed Rust entry
point. Each rebuilds the normal debug APK.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent
MISSING = 'Production native declarations missing from normal APK:'
UNREFERENCED = 'JNI exports in normal APK that no production Kotlin declares:'
NO_ENTRY_POINT = 'Production native declarations with no Rust entry point in awnative:'
MISMATCHED = 'Rust entry points whose JNI types do not match the JVM descriptor:'
KOTLIN = 'lib/rust/src/main/kotlin/com/jtn0123/audiowhisper/rust/'
AWNATIVE = 'lib/rust/crates/awnative/src/'


def append_before_last_brace(source, text):
    at = source.rindex('}')
    return source[:at] + text + source[at:]


def replace_once(old, new):
    """A mutation replacing `old`, which must occur exactly once, with `new`."""
    def change(source):
        assert source.count(old) == 1, 'expected exactly one ' + repr(old)
        return source.replace(old, new)
    return change


# name: ([(file, mutate the source)], lines the failing check must print).
CASES = {
    'declaration-without-export': (
        [(KOTLIN + 'SuggesterNative.kt', lambda source: append_before_last_brace(
            source, '    @JvmStatic external fun mutationMissingExport(): Int\n'))],
        [MISSING, 'Java_com_jtn0123_audiowhisper_rust_SuggesterNative_mutationMissingExport']),
    'export-without-declaration': (
        [(AWNATIVE + 'reader.rs', lambda source: source + '\n#[unsafe(no_mangle)]\npub extern "system" fn '
          'Java_com_jtn0123_audiowhisper_rust_ReaderNative_mutationUndeclared() {}\n')],
        [UNREFERENCED, 'Java_com_jtn0123_audiowhisper_rust_ReaderNative_mutationUndeclared (undeclared)']),
    'test-only-export-shipped': (
        [(AWNATIVE + 'render_theme.rs',
          replace_once('#[cfg(not(target_os = "android"))]\nmod probe {', 'mod probe {'))],
        [UNREFERENCED, 'Java_org_florisboard_lib_snygg_RenderThemeProbe_resolve (test-only)']),
    # KeyModels.select passes a Long; read as a jint, it compiles and links, and loses the top half.
    'entry-point-long-as-int': (
        [(AWNATIVE + 'key_models.rs', replace_once(
            '    kind: jint,\n    raw: jlong,\n    present: jint,\n',
            '    kind: jint,\n    raw: jint,\n    present: jint,\n'))],
        [MISMATCHED, 'Java_com_jtn0123_audiowhisper_rust_KeyModels_select (' + AWNATIVE + 'key_models.rs:',
         '): raw is jint, static (IJI)I needs jlong']),
    # A generated declaration's type changed on one side only. Gradle can't build that for real:
    # both sides regenerate from the same table, and it won't package with cargoBuildAndroid
    # skipped. So the two runs of the generator are made to disagree: `cargo run` for the Kotlin
    # uses the dev profile, with debug assertions, and awnative's release build script does not.
    # The Kotlin (and its caller) take the new Int; the library keeps the Long.
    'generated-declaration-one-side': (
        [('lib/rust/crates/boundary/src/declarations/shift.rs', replace_once(
            '("timeout", Param::Long)',
            '("timeout", if cfg!(debug_assertions) { Param::Int } else { Param::Long })')),
         (KOTLIN + 'Shift.kt', replace_once(
             'ShiftNative.isConsecutive(lastCode, lastTime, code, now, timeout)',
             'ShiftNative.isConsecutive(lastCode, lastTime, code, now, timeout.toInt())'))],
        [MISMATCHED, 'Java_com_jtn0123_audiowhisper_rust_ShiftNative_isConsecutive '
         '(lib/rust/build/rust/boundary/arm64-v8a.rs:', '): timeout is jlong, static (IJIJI)Z needs jint']),
    'entry-point-renamed': (
        [(AWNATIVE + 'clipboard.rs', replace_once(
            'fn Java_com_jtn0123_audiowhisper_rust_ClipboardRules_mimeMatches(',
            'fn Java_com_jtn0123_audiowhisper_rust_ClipboardRules_mimeTypesMatch('))],
        [MISSING, NO_ENTRY_POINT, UNREFERENCED,
         'Java_com_jtn0123_audiowhisper_rust_ClipboardRules_mimeMatches (' + KOTLIN + 'ClipboardRules.kt)',
         'Java_com_jtn0123_audiowhisper_rust_ClipboardRules_mimeTypesMatch (undeclared)']),
}


def run(output, phase):
    with (output / (phase + '.log')).open('wb') as log:
        return subprocess.run(['./gradlew', '--stacktrace', ':app:checkNativeDeclarations'],
                              cwd=ROOT, stdout=log, stderr=subprocess.STDOUT).returncode


def mutate(output, name):
    edits, expected = CASES[name]
    originals = {relative: (ROOT / relative).read_bytes() for relative, _ in edits}
    try:
        for relative, change in edits:
            source = originals[relative].decode()
            mutated = change(source)
            assert mutated != source, name + ': the mutation must change ' + relative
            (ROOT / relative).write_text(mutated)
        mutant = run(output, name)
        log = (output / (name + '.log')).read_text()
        assert mutant != 0, name + ': the gate must fail'
        assert '> Task :app:checkNativeDeclarations FAILED' in log, name + ': the check itself must fail'
        for line in expected:
            assert line in log, name + ': missing ' + line
    finally:
        for relative, original in originals.items():
            (ROOT / relative).write_bytes(original)
    for relative, original in originals.items():
        assert (ROOT / relative).read_bytes() == original, name + ': byte restoration failed for ' + relative
    restored = run(output, name + '-restored')
    assert restored == 0, name + ': the restored source must pass'
    digest = hashlib.sha256(b''.join(originals[relative] for relative in sorted(originals)))
    return {'mutation': name, 'files': sorted(originals), 'mutant_exit': mutant, 'restored_exit': restored,
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
