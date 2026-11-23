"""Compose-free render path faults: each must fail `:app:checkComposeFreeRender` with a fresh problem
naming the class it leaks from, after the mutated source compiled; source bytes are always restored.
python3 -I.

    python3 -I scripts/compose-free-mutations.py build/compose-free/mutations [case ...]

A case is one or more edits: a file, the text it must hold exactly once, and what replaces it; an
empty `old` creates the file, which is deleted again. Run with no other Gradle operation in this
checkout, and do not read the source as the truth while it runs: it is mutated then. A compilation
failure never counts as caught: the check task itself must fail, with the case's marker in a
problem line.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True  # importing mutation_cases would otherwise leave a __pycache__ in scripts/
sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutation_cases

ROOT = Path(__file__).resolve().parent.parent
RENDER = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/render'
ROOT_VIEW = RENDER / 'ImeRootView.kt'
READS = RENDER / 'ImeRootReads.kt'
THEME = RENDER / 'RendererTheme.kt'
KEYBOARD = RENDER / 'RustKeyboardView.kt'
LEAK = RENDER / 'ComposeLeak.kt'
CHECK = ROOT / 'scripts/compose-free-check.py'
GRADLE = ROOT / 'app/build.gradle.kts'
RTL = 'if (reads.keyboard.activeState.isRtl != rtl) reads.keyboard.activeState.isRtl = rtl\n'
COMPOSE_RTL = ('if (reads.keyboard.activeState.isRtl != rtl) reads.keyboard.activeState.layoutDirection =\n'
               '            if (rtl) androidx.compose.ui.unit.LayoutDirection.Rtl '
               'else androidx.compose.ui.unit.LayoutDirection.Ltr\n')
SYSTEM = 'val system = service.resources.getColor(android.R.color.system_accent1_500, service.theme)\n'
# ImeRootReads.kt reading the system accent through a Compose Color.
READS_COLOUR = (READS, SYSTEM, SYSTEM.replace(
    'service.resources.getColor(android.R.color.system_accent1_500, service.theme)',
    'androidx.compose.ui.graphics.Color(service.resources.getColor(android.R.color.system_accent1_500, '
    'service.theme)).hashCode()'))
# Each case: name, [(file, old, new)], the text a problem line must hold.
CASES = [
    # The plain-View root, straight and through a type other code owns.
    ('root-layout-direction', [(ROOT_VIEW, RTL, COMPOSE_RTL)], 'ImeRootView.class: androidx/compose/ui/unit/'),
    ('root-reads-compose-colour', [READS_COLOUR], 'RootReads.class: androidx/compose/ui/graphics/Color'),
    ('root-object-in-rebuild', [(ROOT_VIEW, RTL, RTL + '        post(object : Runnable {\n'
                                 '            override fun run() { androidx.compose.ui.unit.IntSize(1, 1).hashCode() }\n'
                                 '        })\n')], 'ImeRootView$'),
    # The palette and theme the renderer draws in.
    ('theme-mutable-state', [(THEME, 'val names = answer.size / 3',
                              'val names = androidx.compose.runtime.mutableIntStateOf(answer.size / 3).intValue')],
     'RendererThemeKt.class: androidx/compose/runtime/'),
    ('theme-compose-signature', [(THEME, '/** The inline autofill chips',
                                  'internal fun composeSized(size: androidx.compose.ui.unit.IntSize?) = size == null\n\n'
                                  '/** The inline autofill chips')], 'RendererThemeKt.class: androidx/compose/ui/unit/IntSize'),
    # A key's centre through FlorisRect's Compose Offset, as before this path was cleaned.
    ('key-centre-offset', [(KEYBOARD, 'v.left + v.width / 2.0f, v.top + v.height / 2.0f', 'v.center.x, v.center.y')],
     'RustKeyboardView.class: androidx/compose/ui/geometry/Offset'),
    # A new file in the package is checked without being listed anywhere.
    ('new-file-leaks', [(LEAK, '', 'package com.jtn0123.audiowhisper.render\n\n'
                         'internal fun composeLeak() = androidx.compose.ui.unit.Dp(1f).hashCode()\n')],
     'ComposeLeakKt.class: androidx/compose/ui/unit/Dp'),
    # The root's files cannot be excluded, even one whose class has another name (ImeRootReads.kt
    # compiles to RootReads), and the task never passes on a directory with nothing in it.
    ('root-excluded', [(ROOT_VIEW, RTL, COMPOSE_RTL),
                       (CHECK, 'EXCLUDED: dict[str, str] = {}', "EXCLUDED: dict[str, str] = {'ImeRootView.kt': 'fallback'}")],
     'the ImeRoot files are always checked'),
    ('root-reads-excluded', [READS_COLOUR,
                             (CHECK, 'EXCLUDED: dict[str, str] = {}', "EXCLUDED: dict[str, str] = {'ImeRootReads.kt': 'fallback'}")],
     'the ImeRoot files are always checked'),
    ('classes-dir-moved', [(GRADLE, '"app/build/intermediates/built_in_kotlinc/debug/compileDebugKotlin/classes",',
                            '"app/build/intermediates/built_in_kotlinc/debug/compileDebugKotlin/class",')],
     'no such directory'),
]


def check_edits():
    return [mutation_cases.Edit(name, path, old, new) for name, edits, _ in CASES for path, old, new in edits]


def env():
    e = dict(os.environ)
    e.setdefault('ANDROID_HOME', str(Path.home() / 'Library/Android/sdk'))
    return e


def check(log_path):
    """`:app:checkComposeFreeRender`'s exit status, whether the check task itself failed, and its log."""
    with log_path.open('wb') as log:
        result = subprocess.run(['./gradlew', '--console=plain', ':app:checkComposeFreeRender'],
                                cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=env())
    text = log_path.read_text(errors='replace')
    return result.returncode, '> Task :app:checkComposeFreeRender FAILED' in text, text


def apply(edits, originals):
    for path, old, new in edits:
        source = originals[path].decode() if originals[path] is not None else ''
        if old:
            assert source.count(old) == 1, f'mutation location changed: {path.name}: {old!r}'
            path.write_text(source.replace(old, new))
        else:
            assert originals[path] is None, f'{path.name} exists'
            path.write_text(new)


def restore(originals):
    for path, original in originals.items():
        if original is None:
            path.unlink(missing_ok=True)
        else:
            path.write_bytes(original)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('cases', nargs='*', help='case names to run (default: all)')
    args = parser.parse_args()
    unknown = set(args.cases) - {case[0] for case in CASES}
    assert not unknown, f'unknown cases: {sorted(unknown)}'
    cases = [case for case in CASES if not args.cases or case[0] in args.cases]
    output = ROOT / args.output
    output.mkdir(parents=True, exist_ok=False)
    paths = {path for _, edits, _ in cases for path, _, _ in edits}
    originals = {path: path.read_bytes() if path.exists() else None for path in paths}
    status, failed, _ = check(output / 'baseline.log')
    assert status == 0 and not failed, 'the unmutated source must pass the check first'
    results = []
    try:
        for name, edits, marker in cases:
            apply(edits, originals)
            try:
                status, failed, text = check(output / (name + '.log'))
                problems = [line.strip() for line in text.splitlines() if marker in line]
                assert status != 0 and failed and problems, f'{name} must fail the check with {marker!r}'
                results.append({'mutation': name, 'problems': problems})
            finally:
                restore(originals)
            print(f'{name}: caught ({problems[0]})', flush=True)
    finally:
        restore(originals)
    status, failed, _ = check(output / 'restored.log')
    assert status == 0 and not failed, 'restored source fails the check'
    report = {'mutations': results, 'restored_exit': status,
              'source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                                for p in sorted(paths) if p.exists()}}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'All {len(results)} mutations caught; restored source passes the check', flush=True)


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
