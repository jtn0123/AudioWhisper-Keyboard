"""Require every production Kotlin native declaration to be generated (lib/rust/crates/boundary)
and to exist in the packaged debug ELF, every JNI export in it to have one, and every Rust entry
point for one to take and return the JNI types its JVM descriptor maps to.

Scan source, not R8's surviving methods: unused declarations are still dangerous APIs.
Unsupported declarations fail closed rather than disappearing from the inventory. An export no
production source declares is reported as test-only when only test sources (src/test,
src/androidTest, src/testFixtures) declare it, which belongs off the phone (`cfg(not(target_os =
"android"))` or a fixture feature), or as undeclared when no Kotlin does, which is dead code. Both
fail. Fixture builds add exports their tests declare, so only the normal APK is checked.

Names alone miss a Rust entry point that takes a jint where Kotlin passes a Long: the symbol still
links, and the argument arrives cut short. So the debug classes each --module compiled give every
production native method's JVM descriptor, and each Rust entry point with its name, hand-written
in awnative's source or generated (--generated-rust, the copy the APK's build compiled), must
spell the JNI types `javac -h` would give it. The classes must declare the same native methods the
sources do (with --generated-kotlin, the generated ones), so none escapes the check.
"""
import argparse
from collections import Counter
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parent.parent
RUST_SOURCES = ROOT / 'lib/rust/crates/awnative/src'
# Where a module's debug classes are: AGP's built-in Kotlin and javac, then a plain JVM module's.
CLASS_DIRS = ('build/intermediates/built_in_kotlinc/debug/compileDebugKotlin/classes',
              'build/intermediates/javac/debug/compileDebugJavaWithJavac/classes',
              'build/classes/kotlin/main', 'build/classes/java/main')
MISSING = 'Production native declarations missing from normal APK:'
HAND_WRITTEN = 'Production native declarations written by hand; declare them in lib/rust/crates/boundary:'
UNREFERENCED = 'JNI exports in normal APK that no production Kotlin declares:'
UNCOMPILED = 'Production native declarations and compiled classes disagree:'
NO_ENTRY_POINT = 'Production native declarations with no Rust entry point in awnative:'
MISMATCHED = 'Rust entry points whose JNI types do not match the JVM descriptor:'


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


def relative(path):
    return str(path.relative_to(ROOT))


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
            found.append(('Java_' + escape(owner) + '_' + escape(name), relative(path)))
    return found


# The bytes after each constant pool tag but Utf8 (1), which gives its own length.
CONSTANT_SIZES = {3: 4, 4: 4, 5: 8, 6: 8, 7: 2, 8: 2, 9: 4, 10: 4, 11: 4, 12: 4, 15: 3, 16: 2,
                  17: 4, 18: 4, 19: 2, 20: 2}
NATIVE, STATIC = 0x0100, 0x0008


def native_methods(data):
    """A class file's binary name and its native methods, as (name, descriptor, static)."""
    if data[:4] != b'\xca\xfe\xba\xbe':
        raise ValueError('not a class file')
    count, = struct.unpack_from('>H', data, 8)
    strings, classes, at, index = {}, {}, 10, 1
    while index < count:
        tag = data[at]
        if tag == 1:
            length, = struct.unpack_from('>H', data, at + 1)
            # Modified UTF-8; the names that matter here are plain ASCII.
            strings[index] = data[at + 3:at + 3 + length].decode('utf-8', 'replace')
            at += 3 + length
        else:
            if tag == 7:
                classes[index], = struct.unpack_from('>H', data, at + 1)
            at += 1 + CONSTANT_SIZES[tag]
        index += 2 if tag in (5, 6) else 1
    this, = struct.unpack_from('>H', data, at + 2)
    interfaces, = struct.unpack_from('>H', data, at + 6)
    at += 8 + 2 * interfaces
    found = []
    for kind in ('fields', 'methods'):
        members, = struct.unpack_from('>H', data, at)
        at += 2
        for _ in range(members):
            flags, name, descriptor, attributes = struct.unpack_from('>HHHH', data, at)
            at += 8
            for _ in range(attributes):
                at += 6 + struct.unpack_from('>I', data, at + 2)[0]
            if kind == 'methods' and flags & NATIVE:
                found.append((strings[name], strings[descriptor], bool(flags & STATIC)))
    return strings[classes[this]], found


def compiled(modules):
    """Every native method in the modules' debug classes, by JNI name: (descriptor, static, file)."""
    found = {}
    for module in modules:
        for directory in CLASS_DIRS:
            for path in sorted((ROOT / module / directory).rglob('*.class')):
                try:
                    binary, methods = native_methods(path.read_bytes())
                except (ValueError, KeyError, IndexError, struct.error) as error:
                    raise ValueError(f'{relative(path)}: unreadable class file ({error!r})') from error
                for name, descriptor, static in methods:
                    if not re.fullmatch(r'[\w/$]+', binary) or not re.fullmatch(r'\w+', name):
                        raise ValueError(f'{relative(path)}: unsupported native method {name}')
                    symbol = 'Java_' + escape(binary) + '_' + escape(name)
                    found.setdefault(symbol, []).append((descriptor, static, relative(path)))
    return found


def split_parameters(text):
    """`text`, a Rust parameter list, split at its top-level commas."""
    parts, depth, start = [], 0, 0
    for at, c in enumerate(text):
        depth += 1 if c in '(<[' else -1 if c in ')>]' and text[at - 1] != '-' else 0
        if c == ',' and depth == 0:
            parts.append(text[start:at])
            start = at + 1
    parts.append(text[start:])
    return [part.strip() for part in parts if part.strip()]


# jni_sys's primitive types are aliases, so `f32` is a `jfloat`; `bool` is not a `jboolean`.
ALIASES = {'u8': 'jboolean', 'i8': 'jbyte', 'u16': 'jchar', 'i16': 'jshort', 'i32': 'jint',
           'jsize': 'jint', 'i64': 'jlong', 'f32': 'jfloat', 'f64': 'jdouble'}


def rust_type(text):
    """A Rust type as `javac -h` would spell it: paths dropped, spaces single, aliases resolved."""
    spelled = ' '.join(re.sub(r'\b(?:\w+::)+', '', text).split())
    return ALIASES.get(spelled, spelled)


ENTRY_POINT = re.compile(r'extern\s+"system"\s+fn\s+(Java_\w+)\s*\(')


def entry_points(paths):
    """Every Rust JNI entry point in `paths`, by name: ([(parameter, type)], return type, where)."""
    found = {}
    for path in paths:
        source = path.read_text()
        for match in ENTRY_POINT.finditer(source):
            depth, at = 1, match.end()
            while depth:
                depth += {'(': 1, ')': -1}.get(source[at], 0)
                at += 1
            parameters = []
            for parameter in split_parameters(source[match.end():at - 1]):
                typed = re.fullmatch(r'(?:mut\s+)?(\w+)\s*:(?!:)\s*(.+)', parameter, re.S)
                if not typed:
                    raise ValueError(f'{relative(path)}: unsupported parameter {parameter!r}')
                parameters.append((typed.group(1), rust_type(typed.group(2))))
            tail = re.match(r'\s*(?:->\s*([^{]+?))?\s*(?:where\b|\{)', source[at:])
            if not tail:
                raise ValueError(f'{relative(path)}: unsupported entry point {match.group(1)}')
            returns = rust_type(tail.group(1)) if tail.group(1) else '()'
            line = source.count('\n', 0, match.start()) + 1
            found.setdefault(match.group(1), []).append((parameters, returns, f'{relative(path)}:{line}'))
    return found


PRIMITIVES = {'Z': 'jboolean', 'B': 'jbyte', 'C': 'jchar', 'S': 'jshort', 'I': 'jint', 'J': 'jlong',
              'F': 'jfloat', 'D': 'jdouble'}
OBJECTS = {'Ljava/lang/String;': 'jstring', 'Ljava/lang/Class;': 'jclass',
           'Ljava/lang/Throwable;': 'jthrowable'}


def jni_type(descriptor):
    """The JNI type `javac -h` gives a field descriptor."""
    if descriptor in PRIMITIVES:
        return PRIMITIVES[descriptor]
    if descriptor.startswith('['):
        element = descriptor[1:]
        return PRIMITIVES[element] + 'Array' if element in PRIMITIVES else 'jobjectArray'
    return OBJECTS.get(descriptor, 'jobject')


def jni_signature(descriptor, static):
    """The Rust parameter and return types a native method of `descriptor` needs."""
    match = re.fullmatch(r'\(((?:\[*(?:[ZBCSIJFD]|L[^;]+;))*)\)(V|\[*(?:[ZBCSIJFD]|L[^;]+;))',
                         descriptor)
    if not match:
        raise ValueError('unsupported descriptor ' + descriptor)
    parameters = re.findall(r'\[*(?:[ZBCSIJFD]|L[^;]+;)', match.group(1))
    returns = '()' if match.group(2) == 'V' else jni_type(match.group(2))
    return ['*mut JNIEnv', 'jclass' if static else 'jobject'] + [jni_type(p) for p in parameters], returns


def mismatches(name, descriptor, static, points):
    """What is wrong with each of `points`, the entry points named `name`."""
    wanted, returns = jni_signature(descriptor, static)
    kind = ('static ' if static else '') + descriptor
    found = []
    for parameters, actual, where in points:
        if len(parameters) != len(wanted):
            found.append(f'{name} ({where}): takes {len(parameters)} parameters, {kind} needs '
                         f'{len(wanted)}: {", ".join(wanted)}')
            continue
        for (parameter, have), want in zip(parameters, wanted):
            if have != want:
                found.append(f'{name} ({where}): {parameter} is {have}, {kind} needs {want}')
        if actual != returns:
            found.append(f'{name} ({where}): returns {actual}, {kind} needs {returns}')
    return found


def symbols_in(apk):
    sdk = Path(os.environ.get('ANDROID_HOME', Path.home() / 'Library/Android/sdk'))
    candidates = sorted((sdk / 'ndk').glob('*/toolchains/llvm/prebuilt/*/bin/llvm-nm'))
    if not candidates:
        raise RuntimeError('NDK llvm-nm is required')
    with zipfile.ZipFile(apk) as archive, tempfile.TemporaryDirectory() as temporary:
        binary = Path(temporary) / 'libawnative.so'
        binary.write_bytes(archive.read('lib/arm64-v8a/libawnative.so'))
        raw = subprocess.check_output([str(candidates[-1]), '-D', '--defined-only', str(binary)], text=True)
    return {line.split()[-1] for line in raw.splitlines() if line.split()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('apk', type=Path)
    parser.add_argument('--module', action='append', default=[],
                        help='a module directory whose debug classes to read')
    parser.add_argument('--generated-kotlin', action='append', default=[], type=Path,
                        help='a generated production Kotlin source root')
    parser.add_argument('--generated-rust', action='append', default=[], type=Path,
                        help='generated Rust entry points the APK compiled')
    args = parser.parse_args()
    paths = subprocess.check_output(['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z'],
                                    cwd=ROOT).decode().split('\0')
    entries = []
    hand_written = []
    tests = set()
    for path in paths:
        if not path.endswith('.kt'):
            continue
        if any(f'/src/{kind}/' in path for kind in ('main', 'debug')):
            hand_written.extend(declarations(ROOT / path))
        elif any(f'/src/{kind}/' in path for kind in ('test', 'androidTest', 'testFixtures')):
            tests.update(name for name, _ in declarations(ROOT / path))
    entries.extend(hand_written)
    for root in args.generated_kotlin:
        for path in sorted((ROOT / root).rglob('*.kt')):
            entries.extend(declarations(path))
    duplicates = [name for name, count in Counter(name for name, _ in entries).items() if count > 1]
    classes = compiled(args.module)
    duplicates += [name for name, methods in classes.items() if len(methods) > 1 and name not in duplicates]
    if duplicates:
        raise ValueError('Overloaded native declarations need signature checking: ' + ', '.join(duplicates))
    symbols = symbols_in(args.apk)
    errors = [(HAND_WRITTEN, [f'{name} ({path})' for name, path in hand_written])]
    missing = [(name, path) for name, path in entries if name not in symbols]
    errors.append((MISSING, [f'{name} ({path})' for name, path in missing]))
    exports = sorted(name for name in symbols if name.startswith('Java_'))
    declared = {name for name, _ in entries}
    errors.append((UNREFERENCED, [f'{name} ({"test-only" if name in tests else "undeclared"})'
                                  for name in exports if name not in declared]))
    errors.append((UNCOMPILED, [f'{name} ({path}): no compiled class declares it'
                                for name, path in entries if name not in classes] +
                   [f'{name} ({methods[0][2]}): no production source declares it'
                    for name, methods in sorted(classes.items()) if name not in declared]))
    sources = sorted(RUST_SOURCES.rglob('*.rs')) + [ROOT / path for path in args.generated_rust]
    points = entry_points(sources)
    checked = [name for name, _ in entries if name in classes and name in points]
    errors.append((NO_ENTRY_POINT, [f'{name} ({path})' for name, path in entries if name not in points]))
    errors.append((MISMATCHED, [line for name in checked
                                for line in mismatches(name, *classes[name][0][:2], points[name])]))
    report = [headline + '\n' + '\n'.join(lines) for headline, lines in errors if lines]
    if report:
        raise RuntimeError('\n\n'.join(report))
    print(json.dumps({'production_native_declarations': len(entries), 'hand_written': 0, 'missing': 0,
                      'jni_exports': len(exports), 'test_only': 0, 'undeclared': 0,
                      'descriptors_checked': len(checked),
                      'rust_entry_points_checked': sum(len(points[name]) for name in checked),
                      'mismatched': 0}))


if __name__ == '__main__':
    main()
