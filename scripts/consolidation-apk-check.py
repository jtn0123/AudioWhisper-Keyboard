"""Check the actual packaged native payload, including fixture-mode/cache identity."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import zipfile

PREFIX = 'Java_com_jtn0123_audiowhisper_rust_'
LEGACY = ['SuggesterNative_' + method for method in ('suggest', 'spell', 'learnerFinished', 'learnerSwiped')]
LEGACY += ['DictationNative_join']
SNAPSHOT = 'Java_com_jtn0123_audiowhisper_suggest_A1Probe_snapshot'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('apk', type=Path)
    parser.add_argument('--fixture', action='store_true')
    parser.add_argument('--compiled', type=Path)
    args = parser.parse_args()
    sdk = Path.home() / 'Library/Android/sdk'
    nm = next((sdk / 'ndk').glob('*/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-nm'))
    with zipfile.ZipFile(args.apk) as archive, tempfile.TemporaryDirectory() as temporary:
        data = archive.read('lib/arm64-v8a/libawnative.so')
        path = Path(temporary) / 'libawnative.so'
        path.write_bytes(data)
        raw = subprocess.check_output([str(nm), '-D', '--defined-only', str(path)], text=True)
        symbols = {line.split()[-1] for line in raw.splitlines() if line.split()}
    if (SNAPSHOT in symbols) != args.fixture:
        raise RuntimeError('Packaged A1 fixture mode differs from the requested mode')
    legacy = [name for name in LEGACY if PREFIX + name in symbols]
    expected_legacy = len(LEGACY) if args.fixture else 0
    if len(legacy) != expected_legacy:
        raise RuntimeError('Packaged legacy callback exports differ from the requested mode')
    if args.compiled and data != args.compiled.read_bytes():
        raise RuntimeError('Packaged native bytes differ from the compiled artifact')
    for name in ('NativeLearnerNative_call', 'NativeGlideNative_rank', 'ReaderWalk_walk'):
        if PREFIX + name not in symbols:
            raise RuntimeError('Required engine export missing: ' + name)
    print(json.dumps({'native_sha256': hashlib.sha256(data).hexdigest(),
                      'fixture': args.fixture, 'legacy_exports': len(legacy)}))


if __name__ == '__main__':
    main()
