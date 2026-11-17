"""Break the native boundary three ways; require the APK gate to reject each, then pass restored.

python3 -I scripts/native-declarations-mutation.py OUTPUT [CASE...]
The cases: a production Kotlin declaration with no export, an export no Kotlin declares, and an
export only tests declare left in the phone's library. Each rebuilds the normal debug APK.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent
MISSING = 'Production native declarations missing from normal APK:'
UNREFERENCED = 'JNI exports in normal APK that no production Kotlin declares:'


def append_before_last_brace(source, text):
    at = source.rindex('}')
    return source[:at] + text + source[at:]


# name: (file, mutate the source, lines the failing check must print).
CASES = {
    'declaration-without-export': (
        'lib/rust/src/main/kotlin/com/jtn0123/audiowhisper/rust/SuggesterNative.kt',
        lambda source: append_before_last_brace(
            source, '    @JvmStatic external fun mutationMissingExport(): Int\n'),
        [MISSING, 'Java_com_jtn0123_audiowhisper_rust_SuggesterNative_mutationMissingExport']),
    'export-without-declaration': (
        'lib/rust/crates/awnative/src/reader.rs',
        lambda source: source + '\n#[unsafe(no_mangle)]\npub extern "system" fn '
                                'Java_com_jtn0123_audiowhisper_rust_ReaderNative_mutationUndeclared() {}\n',
        [UNREFERENCED, 'Java_com_jtn0123_audiowhisper_rust_ReaderNative_mutationUndeclared (undeclared)']),
    'test-only-export-shipped': (
        'lib/rust/crates/awnative/src/render_theme.rs',
        lambda source: source.replace('#[cfg(not(target_os = "android"))]\nmod probe {', 'mod probe {', 1),
        [UNREFERENCED, 'Java_org_florisboard_lib_snygg_RenderThemeProbe_resolve (test-only)']),
}


def run(output, phase):
    with (output / (phase + '.log')).open('wb') as log:
        return subprocess.run(['./gradlew', '--stacktrace', ':app:checkNativeDeclarations'],
                              cwd=ROOT, stdout=log, stderr=subprocess.STDOUT).returncode


def mutate(output, name):
    relative, change, expected = CASES[name]
    path = ROOT / relative
    original = path.read_bytes()
    try:
        source = original.decode()
        mutated = change(source)
        assert mutated != source, name + ': the mutation must change the source'
        path.write_text(mutated)
        mutant = run(output, name)
        log = (output / (name + '.log')).read_text()
        assert mutant != 0, name + ': the gate must fail'
        assert '> Task :app:checkNativeDeclarations FAILED' in log, name + ': the check itself must fail'
        for line in expected:
            assert line in log, name + ': missing ' + line
    finally:
        path.write_bytes(original)
    assert path.read_bytes() == original, name + ': byte restoration failed'
    restored = run(output, name + '-restored')
    assert restored == 0, name + ': the restored source must pass'
    return {'mutation': name, 'mutant_exit': mutant, 'restored_exit': restored,
            'restored_source_sha256': hashlib.sha256(original).hexdigest()}


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
