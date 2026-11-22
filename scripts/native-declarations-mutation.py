"""Break the native boundary; require the APK gate to reject each break, then pass restored.

python3 -I scripts/native-declarations-mutation.py OUTPUT [CASE...]
The cases: a hand-written production Kotlin declaration, an export no Kotlin declares, an export
only tests declare left in the phone's library, a generated declaration whose type changed on one
side only, a generated method renamed on one side only, and one per kind of generated entry point
whose JNI types javac would spell differently: a callback object, an array of objects, an array of
int arrays, a char array, and an instance method. Each rebuilds the normal debug APK.
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
BOUNDARY = 'lib/rust/crates/boundary/src/'
HAND_WRITTEN = 'Production native declarations written by hand; declare them in lib/rust/crates/boundary:'
# The generated entry points the APK compiled, which mismatches name.
GENERATED = '(lib/rust/build/rust/boundary/arm64-v8a.rs:'
JNI = 'Java_com_jtn0123_audiowhisper_rust_'


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
    'declaration-hand-written': (
        [(KOTLIN + 'Gif.kt', lambda source: source + (
            '\ninternal object MutationNative {\n'
            '    @JvmStatic external fun mutationHandWritten(): Int\n}\n'))],
        [HAND_WRITTEN, MISSING, NO_ENTRY_POINT,
         JNI + 'MutationNative_mutationHandWritten (' + KOTLIN + 'Gif.kt)']),
    'export-without-declaration': (
        [(AWNATIVE + 'reader.rs', lambda source: source + '\n#[unsafe(no_mangle)]\npub extern "system" fn '
          'Java_com_jtn0123_audiowhisper_rust_ReaderNative_mutationUndeclared() {}\n')],
        [UNREFERENCED, JNI + 'ReaderNative_mutationUndeclared (undeclared)']),
    'test-only-export-shipped': (
        [(AWNATIVE + 'render_theme.rs',
          replace_once('#[cfg(not(target_os = "android"))]\nmod probe {', 'mod probe {'))],
        [UNREFERENCED, 'Java_org_florisboard_lib_snygg_RenderThemeProbe_resolve (test-only)']),
    # A generated declaration's type changed on one side only. Gradle can't build that for real:
    # both sides regenerate from the same table, and it won't package with cargoBuildAndroid
    # skipped. So the two runs of the generator are made to disagree: `cargo run` for the Kotlin
    # uses the dev profile, with debug assertions, and awnative's release build script does not.
    # The Kotlin (and its hand-written caller) take the new Int; the library keeps the Long.
    'generated-declaration-one-side': (
        [(BOUNDARY + 'declarations/editor.rs', replace_once(
            'method("free", &[HANDLE_PARAM], Return::Unit)',
            'method("free", &[("handle", if cfg!(debug_assertions) { Param::Int } else { Param::Long })], '
            'Return::Unit)')),
         (KOTLIN + 'EditorSession.kt', replace_once(
             'EditorNative.free(owned)', 'EditorNative.free(owned.toInt())'))],
        [MISMATCHED, JNI + 'EditorNative_free ' + GENERATED, '): handle is jlong, static (I)V needs jint']),
    # The same trick renames a method in the library only; its `rust` path keeps it compiling.
    'generated-method-renamed': (
        [(BOUNDARY + 'declarations/clipboard.rs', replace_once(
            'name: "mimeMatches",',
            'name: if cfg!(debug_assertions) { "mimeMatches" } else { "mimeTypesMatch" },'))],
        [MISSING, NO_ENTRY_POINT, UNREFERENCED,
         JNI + 'ClipboardRules_mimeMatches (lib/rust/build/generated/nativeBoundary/kotlin/',
         JNI + 'ClipboardRules_mimeTypesMatch (undeclared)']),
    # Each of these spells one kind of JNI type differently in the generated Rust only. Every JNI
    # reference type is an alias of jobject, so the library still compiles and links, and the
    # Kotlin is untouched: only the check can tell.
    'callback-object-as-string': (
        [(BOUNDARY + 'types.rs', replace_once(
            'Param::Object(_) => "jobject",', 'Param::Object(_) => "jstring",'))],
        [MISMATCHED, JNI + 'TtsNative_generate ' + GENERATED,
         '): callback is jstring, static (JLjava/lang/String;Lcom/jtn0123/audiowhisper/rust/TtsAudio;)V '
         'needs jobject']),
    'objects-as-object': (
        [(BOUNDARY + 'api/records.rs', replace_once(
            'Return::Object("Array<Any>", "jobjectArray");', 'Return::Object("Array<Any>", "jobject");'))],
        [MISMATCHED, JNI + 'GlideNative_score ' + GENERATED, ')[Ljava/lang/Object; needs jobjectArray']),
    'int-arrays-as-int-array': (
        [(BOUNDARY + 'types.rs', replace_once(
            'Return::Strings | Return::IntArrays | Return::StringArrays => "jobjectArray",',
            'Return::Strings | Return::StringArrays => "jobjectArray",\n'
            '            Return::IntArrays => "jintArray",'))],
        [MISMATCHED, JNI + 'ClipboardRules_plan ' + GENERATED,
         '): returns jintArray, ([J[Ljava/lang/String;[J)[[I needs jobjectArray']),
    'chars-as-shorts': (
        [(BOUNDARY + 'types.rs', replace_once(
            'Element::Char => "jcharArray",', 'Element::Char => "jshortArray",'))],
        [MISMATCHED, JNI + 'GlideNative_distinctChars ' + GENERATED, '): returns jshortArray, static (',
         ')[C needs jcharArray']),
    'instance-method-as-static': (
        [(BOUNDARY + 'rust.rs', replace_once(
            'let this = if method.instance { "jobject" } else { "jclass" };',
            'let this = if method.instance { "jclass" } else { "jclass" };'))],
        [MISMATCHED, JNI + 'ClipboardRules_mimeMatches ' + GENERATED,
         '): _ is jclass, (Ljava/lang/String;Ljava/lang/String;)Z needs jobject']),
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
