"""Fails when a compiled class of the plain-View render path names Compose.

Every class file under <classes dir>/com/jtn0123/audiowhisper/render is read, nested, lambda and
synthetic classes included, and each string of its constant pool is checked. A class reference, a
field or method descriptor, a generic signature or Kotlin metadata naming a type under
androidx/compose spells that internal name there, so any string holding `androidx/compose/` is a
reference. A string literal spelling one counts too: the check is strict on purpose. The one string
allowed is the Compose compiler plugin's stamp (PLUGIN_STAMP), which no source line writes.

Each class is known by the source file its SourceFile attribute names, so a file's nested, lambda
and differently named classes (ImeRootReads.kt compiles to RootReads) go with it. The ImeRoot*.kt
files, the plain-View root, are always checked. EXCLUDED lists the frozen Compose fallback files in
this package by name; it is empty because the fallback (FlorisImeUi, FlorisImeViews, the Compose
keyboard and Smartbar) lives outside it. A directory with no render class, a run with no class
from an ImeRoot file, or a class with no SourceFile fails: the check never passes on nothing.

    python3 -I -B scripts/compose-free-check.py <classes dir> [<classes dir> ...]
"""
import struct
import sys
from pathlib import Path

PACKAGE = 'com/jtn0123/audiowhisper/render'
COMPOSE = b'androidx/compose/'
ROOT_PREFIX = 'ImeRoot'
# Source file -> why its classes may still name Compose. None: no frozen Compose file is in this package.
EXCLUDED: dict[str, str] = {}
# The Compose compiler plugin, applied to the whole app module, stamps every class it compiles with this
# annotation (binary retention: never loaded at runtime). The stamp is the only Compose name allowed, and only
# as that annotation's descriptor; it goes when step K takes the plugin out of the module.
PLUGIN_STAMP = b'Landroidx/compose/runtime/internal/StabilityInferred;'
# How many bytes each constant-pool tag's entry holds after the tag, and how many slots it takes.
FIXED = {3: (4, 1), 4: (4, 1), 5: (8, 2), 6: (8, 2), 7: (2, 1), 8: (2, 1), 9: (4, 1), 10: (4, 1),
         11: (4, 1), 12: (4, 1), 15: (3, 1), 16: (2, 1), 17: (4, 1), 18: (4, 1), 19: (2, 1), 20: (2, 1)}
UTF8 = 1


def constant_pool(data):
    """Class file `data`'s UTF-8 constants by slot, as bytes, and the offset just after the pool."""
    magic, _, _, count = struct.unpack_from('>IHHH', data, 0)
    if magic != 0xCAFEBABE:
        raise ValueError('not a class file')
    at, slot, found = 10, 1, {}
    while slot < count:
        tag = data[at]
        if tag == UTF8:
            (length,) = struct.unpack_from('>H', data, at + 1)
            found[slot] = data[at + 3:at + 3 + length]
            at, slot = at + 3 + length, slot + 1
        elif tag in FIXED:
            size, slots = FIXED[tag]
            at, slot = at + 1 + size, slot + slots
        else:
            raise ValueError(f'constant pool tag {tag} at byte {at}')
    return found, at


def skip_attributes(data, at):
    """The offset after the attribute table at `at`."""
    (count,) = struct.unpack_from('>H', data, at)
    at += 2
    for _ in range(count):
        (length,) = struct.unpack_from('>I', data, at + 2)
        at += 6 + length
    return at


def source_file(data, pool, at):
    """The file its SourceFile attribute names, for class file `data` whose pool ends at `at`; None without one."""
    (interfaces,) = struct.unpack_from('>H', data, at + 6)
    at += 8 + 2 * interfaces
    for _ in range(2):  # fields, then methods: access, name, descriptor, attributes
        (members,) = struct.unpack_from('>H', data, at)
        at += 2
        for _ in range(members):
            at = skip_attributes(data, at + 6)
    (count,) = struct.unpack_from('>H', data, at)
    at += 2
    for _ in range(count):
        name, length = struct.unpack_from('>HI', data, at)
        if pool.get(name) == b'SourceFile':
            (index,) = struct.unpack_from('>H', data, at + 6)
            return pool[index].decode()
        at += 6 + length
    return None


def compose_names(pool):
    """Each Compose internal name constant pool `pool` spells, sorted."""
    names = set()
    for text in pool.values():
        if text == PLUGIN_STAMP:
            continue
        start = text.find(COMPOSE)
        while start >= 0:
            end = start
            while end < len(text) and (chr(text[end]).isalnum() or text[end] in b'/_$'):
                end += 1
            names.add(text[start:end].decode())
            start = text.find(COMPOSE, end)
    return sorted(names)


def main(argv):
    if not argv:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    bad = [name for name in EXCLUDED if name.startswith(ROOT_PREFIX)]
    if bad:
        print(f'the ImeRoot files are always checked; EXCLUDED names {bad}', file=sys.stderr)
        return 1
    classes, problems = [], []
    for directory in map(Path, argv):
        if not directory.is_dir():
            problems.append(f'{directory}: no such directory')
            continue
        classes += [(directory, c) for c in sorted((directory / PACKAGE).rglob('*.class'))]
    roots = 0
    for directory, path in classes:
        data = path.read_bytes()
        pool, end = constant_pool(data)
        source = source_file(data, pool, end)
        if source is None:
            problems.append(f'{path.relative_to(directory)}: no SourceFile attribute, so no file to check it under')
            continue
        roots += source.startswith(ROOT_PREFIX)
        if source in EXCLUDED:
            continue
        names = compose_names(pool)
        if names:
            problems.append(f'{path.relative_to(directory)}: {", ".join(names)}')
    if not classes:
        problems.append(f'no class under {PACKAGE} in {", ".join(argv)}')
    elif not roots:
        problems.append(f'no class from an {ROOT_PREFIX} file under {PACKAGE}: the plain-View root went unchecked')
    if problems:
        print(f'{len(problems)} problem(s) in the Compose-free render path:', file=sys.stderr)
        for problem in problems:
            print(f'  {problem}', file=sys.stderr)
        return 1
    print(f'{len(classes)} classes in {PACKAGE} ({roots} from ImeRoot files), none naming androidx.compose; '
          f'{len(EXCLUDED)} files excluded')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
