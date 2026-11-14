"""Require every production Kotlin native declaration to exist in the packaged debug ELF.

Scan source, not R8's surviving methods: unused declarations are still dangerous APIs.
Unsupported declarations fail closed rather than disappearing from the inventory.
"""
import argparse
from collections import Counter
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parent.parent


def tokens(source):
    """Kotlin tokens needed for declarations; skip nested comments and all literals."""
    at = 0
    while at < len(source):
        if source.startswith('//', at):
            end = source.find('\n', at)
            at = len(source) if end < 0 else end
        elif source.startswith('/*', at):
            depth = 1
            at += 2
            while depth and at < len(source):
                if source.startswith('/*', at):
                    depth += 1
                    at += 2
                elif source.startswith('*/', at):
                    depth -= 1
                    at += 2
                else:
                    at += 1
            if depth:
                raise ValueError('Unterminated comment')
        elif source.startswith('"""', at):
            end = source.find('"""', at + 3)
            if end < 0:
                raise ValueError('Unterminated raw string')
            at = end + 3
        elif source[at] in '\"\'':
            quote = source[at]
            at += 1
            while at < len(source):
                if source[at] == '\\':
                    at += 2
                elif source[at] == quote:
                    at += 1
                    break
                else:
                    at += 1
        else:
            match = re.match(r'[\w$]+|[^\s]', source[at:])
            if match:
                yield match.group()
                at += len(match.group())
            else:
                at += 1


def escape(name):
    return ''.join('_1' if c == '_' else '_' if c in './' else
                   '_00024' if c == '$' else c for c in name)


def declarations(path):
    source = path.read_text()
    if not re.search(r'\bexternal\s+fun\b', source):
        return []
    stream = list(tokens(source))
    package = re.search(r'^package\s+([\w.]+)', source, re.M)
    if not package:
        raise ValueError(f'{path}: native source must declare its package')
    scopes = []
    pending = None
    found = []
    for i, token in enumerate(stream):
        if token in ('object', 'class', 'interface'):
            name = stream[i + 1]
            pending = name if re.fullmatch(r'\w+', name) else None
        elif token == '{':
            scopes.append(pending)
            pending = None
        elif token == '}':
            scopes.pop()
        elif token == 'external' and stream[i + 1] == 'fun':
            name = stream[i + 2]
            owners = [s for s in scopes if s]
            if not owners or not re.fullmatch(r'\w+', name) or stream[i + 3] != '(':
                raise ValueError(f'{path}: unsupported native declaration near {name}')
            owner = package.group(1) + '.' + '$'.join(owners)
            found.append(('Java_' + escape(owner) + '_' + escape(name), str(path.relative_to(ROOT))))
    return found


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('apk', type=Path)
    args = parser.parse_args()
    paths = subprocess.check_output(['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z'],
                                    cwd=ROOT).decode().split('\0')
    entries = []
    for path in paths:
        if path.endswith('.kt') and any(f'/src/{kind}/' in path for kind in ('main', 'debug')):
            entries.extend(declarations(ROOT / path))
    duplicates = [name for name, count in Counter(name for name, _ in entries).items() if count > 1]
    if duplicates:
        raise ValueError('Overloaded native declarations need signature checking: ' + ', '.join(duplicates))
    sdk = Path(os.environ.get('ANDROID_HOME', Path.home() / 'Library/Android/sdk'))
    candidates = sorted((sdk / 'ndk').glob('*/toolchains/llvm/prebuilt/*/bin/llvm-nm'))
    if not candidates:
        raise RuntimeError('NDK llvm-nm is required')
    with zipfile.ZipFile(args.apk) as apk, tempfile.TemporaryDirectory() as temporary:
        binary = Path(temporary) / 'libawnative.so'
        binary.write_bytes(apk.read('lib/arm64-v8a/libawnative.so'))
        raw = subprocess.check_output([str(candidates[-1]), '-D', '--defined-only', str(binary)], text=True)
    symbols = {line.split()[-1] for line in raw.splitlines() if line.split()}
    missing = [(name, path) for name, path in entries if name not in symbols]
    if missing:
        raise RuntimeError('Production native declarations missing from normal APK:\n' +
                           '\n'.join(f'{name} ({path})' for name, path in missing))
    print(json.dumps({'production_native_declarations': len(entries), 'missing': 0}))


if __name__ == '__main__':
    main()
