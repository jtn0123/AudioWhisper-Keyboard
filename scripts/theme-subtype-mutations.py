"""Theme faults (phase 3 item 3): each must trip a fresh assertion in at least one of the test
targets it names; source bytes are always restored. python3 -I.

    python3 -I scripts/theme-subtype-mutations.py build/theme-subtype/mutations [case ...]
    python3 -I scripts/theme-subtype-mutations.py --check     # only that every location is still there

Run with no other Cargo or Gradle operation in this checkout, and do not read the source as the
truth while it runs: it is mutated then. A compilation failure never counts as caught: a Rust target
needs a "panicked at" line, a JVM target an assertion failure in a fresh test report. Every target
a case names is run and recorded, so the result says which caught it.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import xml.etree.ElementTree as ET

sys.dont_write_bytecode = True  # importing mutation_cases would otherwise leave a __pycache__ in scripts/
sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutation_cases

ROOT = Path(__file__).resolve().parent.parent
THEME_SRC = ROOT / 'lib/rust/crates/imetheme/src'
CHOOSE, SESSION, INLINE, CALLS = (THEME_SRC / name for name in ('choose.rs', 'session.rs', 'inline.rs', 'calls.rs'))
UPDATES = ROOT / 'app/src/main/kotlin/dev/patrickgold/florisboard/ime/theme/ThemeUpdates.kt'
RENDERER = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/render/RendererTheme.kt'
# The targets: the Rust crate's tests, and JVM test classes run by one Gradle call each.
UNIT, TRACE, CHIP = 'cargo:imetheme', 'jvm:theme', 'jvm:chip'
THEME, CHIPS = [UNIT, TRACE], [UNIT, CHIP]
CASES = [
    # Which theme the settings choose.
    ('preview-id-ignored', CHOOSE, 'if let Some(preview) = &self.preview {',
     'if let Some(preview) = self.preview.as_ref().filter(|_| false) {', THEME),
    ('day-and-night-swapped', CHOOSE, 'Mode::AlwaysDay => true,\n            Mode::AlwaysNight => false,',
     'Mode::AlwaysDay => false,\n            Mode::AlwaysNight => true,', THEME),
    ('system-night-ignored', CHOOSE, 'Mode::FollowSystem => !self.system_night,', 'Mode::FollowSystem => true,', THEME),
    ('sunrise-excluded', CHOOSE, 'self.sunrise <= self.now &&', 'self.sunrise < self.now &&', THEME),
    ('sunset-excluded', CHOOSE, '&& self.now <= self.sunset', '&& self.now < self.sunset', THEME),
    ('time-wraps-midnight', CHOOSE, 'Mode::FollowTime => self.sunrise <= self.now && self.now <= self.sunset,',
     'Mode::FollowTime => if self.sunrise <= self.sunset { self.sunrise <= self.now && self.now <= self.sunset } '
     'else { self.now >= self.sunrise || self.now <= self.sunset },', THEME),
    # What an update does: the cache, the preview, the lookups, the load.
    ('abandoned-load-finishes', SESSION, '        self.pending = None;\n', '', THEME),
    ('clear-ignored', SESSION, '        if clear {\n            self.cache.clear();\n        }\n', '', THEME),
    ('preview-info-ignored', SESSION, '        if preview_info {\n            return Step::Preview;\n        }\n', '', THEME),
    ('cache-ignored', SESSION, '.find(|(cached, _)| cached == name)', '.find(|(cached, _)| cached == name && false)', THEME),
    ('keyboard-shadow-ignored', SESSION, 'if index.keyboards.contains(&name.extension) {', 'if false {', THEME),
    ('first-component-loaded', SESSION, '.position(|c| c.id == name.component) else {',
     '.position(|_| true) else {', THEME),
    ('sourceless-loaded', SESSION, 'if !theme.source {', 'if false {', THEME),
    ('own-stylesheet-ignored', SESSION, 'let stylesheet = &theme.components[component].stylesheet;',
     'let stylesheet = &Vec::new();', THEME),
    ('failure-kept', SESSION, 'if !ok {\n            return Some(Step::Failed);', 'if false {\n            return Some(Step::Failed);',
     THEME),
    ('slot-reused', SESSION, '        self.next += 1;\n', '', THEME),
    # The rows that cross from Android and back.
    ('preview-flag-ignored', CALLS, 'let preview = wire::flag::decode(&row[update::PREVIEW]);', 'let preview = false;', THEME),
    ('day-read-as-night', CALLS, 'day: name(update::DAY_EXTENSION, update::DAY_COMPONENT),',
     'day: name(update::NIGHT_EXTENSION, update::NIGHT_COMPONENT),', THEME),
    ('keyboards-dropped', CALLS, 'keyboards: rest[..keyboards].to_vec(),', 'keyboards: vec![],', THEME),
    ('source-flag-ignored', CALLS, 'source: wire::flag::decode(&head[extension::SOURCE]),', 'source: true,', THEME),
    ('clear-not-reported', CALLS, '                clear,\n            ));', '                false,\n            ));', THEME),
    ('primary-always-system', CALLS, 'let chosen = if wire::flag::decode(&row[accent::SPECIFIED]) {', 'let chosen = if false {',
     THEME),
    ('chip-foreground-for-background', CALLS, 'out[chip::BACKGROUND] = number_text(colors.0 as i32);',
     'out[chip::BACKGROUND] = number_text(colors.1 as i32);', CHIPS),
    # The chips' colours.
    ('chip-outside-the-window', INLINE, '&["root", "window", "inline-autofill-chip"]',
     '&["root", "inline-autofill-chip"]', CHIPS),
    ('root-without-attributes', INLINE, 'matches!(*element, "root" | "window")', 'matches!(*element, "window")', CHIPS),
    ('chip-pressed', INLINE, '            selector: 0,', '            selector: 1,', CHIPS),
    ('light-and-dark-swapped', INLINE, '1.0, &light, &dark)', '1.0, &dark, &light)', CHIPS),
    ('white-default-foreground', INLINE, 'argb("foreground", FOREGROUND),', 'argb("foreground", BACKGROUND),', CHIPS),
    ('red-and-blue-swapped', INLINE, 'b[3] << 24 | b[0] << 16 | b[1] << 8 | b[2]', 'b[3] << 24 | b[2] << 16 | b[1] << 8 | b[0]',
     CHIPS),
    # The Kotlin that sends the facts and the index, and holds the loaded themes.
    ('system-night-not-sent', UPDATES, 'ImeThemeUpdate.SYSTEM_NIGHT to WireFlag.encode(facts.systemNight),',
     'ImeThemeUpdate.SYSTEM_NIGHT to WireFlag.encode(false),', [TRACE]),
    ('preview-info-not-sent', UPDATES, 'ImeThemeUpdate.PREVIEW_INFO to WireFlag.encode(previewInfo),',
     'ImeThemeUpdate.PREVIEW_INFO to WireFlag.encode(false),', [TRACE]),
    ('now-sent-as-sunset', UPDATES, 'ImeThemeUpdate.NOW to facts.now.toString(),', 'ImeThemeUpdate.NOW to facts.sunset.toString(),',
     [TRACE]),
    ('archive-always-there', UPDATES, 'ImeThemeExtension.SOURCE to WireFlag.encode(theme.hasSource),',
     'ImeThemeExtension.SOURCE to WireFlag.encode(true),', [TRACE]),
    ('own-stylesheet-not-sent', UPDATES, 'ImeThemeComponent.STYLESHEET to component.stylesheetPath.orEmpty(),',
     'ImeThemeComponent.STYLESHEET to "",', [TRACE]),
    ('cached-not-shown', UPDATES, 'ImeThemeStepKind.CACHED -> loaded.getValue(step[ImeThemeStep.SLOT].toInt())',
     'ImeThemeStepKind.CACHED -> null', [TRACE]),
    ('loaded-named-by-component', UPDATES, 'val name = ExtensionComponentName(source.meta.id, config.id)',
     'val name = ExtensionComponentName(config.id, config.id)', [TRACE]),
    ('failure-without-cause', UPDATES, 'loadFailure = ThemeManager.LoadFailure(source.meta, config, checkNotNull(result.exceptionOrNull())),',
     'loadFailure = ThemeManager.LoadFailure(source.meta, config, IllegalStateException()),', [TRACE]),
    # The accent, the renderer's theme and the chips' palette.
    ('accent-not-sent', RENDERER, 'ImeThemeAccent.PREFERRED to "${accent ?: 0}",', 'ImeThemeAccent.PREFERRED to "0",',
     [TRACE]),
    ('renderer-without-palette', RENDERER, 'RendererTheme(info.stylesheetJson, palette.names, palette.colors,',
     'RendererTheme(info.stylesheetJson, palette.names, palette.colors.map { 0 },', [TRACE]),
    ('chips-without-palette', RENDERER, '*palette.colors.map { it.toString() }.toTypedArray())',
     '*palette.colors.map { "0" }.toTypedArray())', [TRACE]),
    ('chips-in-the-letters-always', RENDERER, 'ImeThemeInline.MODE to mode,', 'ImeThemeInline.MODE to "characters",', [TRACE]),
    ('chips-unshifted-always', RENDERER, 'ImeThemeInline.SHIFT to shift,', 'ImeThemeInline.SHIFT to "unshifted",', [TRACE]),
]
JVM = {TRACE: (':app:testDebugUnitTest', ['*ThemeMatchTest'], ROOT / 'app/build/test-results/testDebugUnitTest'),
       CHIP: (':lib:snygg:testDebugUnitTest', ['*InlineChipMatchTest'], ROOT / 'lib/snygg/build/test-results/testDebugUnitTest')}
CARGO = {UNIT: ['-p', 'imetheme']}


def env():
    e = dict(os.environ)
    e.setdefault('CARGO_BUILD_JOBS', '6')
    e.setdefault('ANDROID_HOME', str(Path.home() / 'Library/Android/sdk'))
    return e


def log_of(output, name, target):
    return output / (name + '.' + target.replace(':', '-') + '.log')


def compiled(log_path):
    """False when the build failed: a mutant that doesn't compile is no test's catch."""
    text = log_path.read_text(errors='replace')
    return not any(sign in text for sign in ('could not compile', '\ne: ', 'Compilation error'))


def rust(output, name, target):
    log_path = log_of(output, name, target)
    with log_path.open('wb') as log:
        result = subprocess.run(['cargo', 'test', '-q'] + CARGO[target], cwd=ROOT / 'lib/rust', stdout=log,
                                stderr=subprocess.STDOUT, env=env())
    text = log_path.read_text(errors='replace')
    return result.returncode, sum('panicked at' in line for line in text.splitlines())


def assertion(failure):
    kind = failure.attrib.get('type', '')
    return 'AssertionFailedError' in kind or kind == 'java.lang.AssertionError'


def jvm(output, name, target):
    started = time.time()
    task, patterns, reports = JVM[target]
    command = ['./gradlew', '--stacktrace', task]
    for pattern in patterns:
        command += ['--tests', pattern]
    with log_of(output, name, target).open('wb') as log:
        result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=env())
    failures = 0
    for report in reports.glob('TEST-*.xml'):
        if report.stat().st_mtime >= started:
            failures += sum(assertion(f) for f in ET.parse(report).getroot().iter('failure'))
    return result.returncode, failures


def run(output, name, target):
    return (rust if target.startswith('cargo:') else jvm)(output, name, target)


def check(cases):
    originals = {path: path.read_bytes() for _, path, _, _, _ in cases}
    for name, path, old, new, _ in cases:
        assert originals[path].decode().count(old) == 1, 'mutation location changed: ' + name
        assert old != new, 'mutation changes nothing: ' + name
    names = [case[0] for case in cases]
    assert len(names) == len(set(names)), 'case names repeat'
    return originals


def mutate(output, cases, originals):
    results = []
    for name, path, old, new, targets in cases:
        path.write_text(originals[path].decode().replace(old, new))
        try:
            outcome = {}
            for target in targets:
                status, failures = run(output, name, target)
                outcome[target] = {'exit': status, 'assertion_failures': failures,
                                   'compiled': compiled(log_of(output, name, target))}
        finally:
            path.write_bytes(originals[path])
        caught = [t for t, o in outcome.items() if o['exit'] != 0 and o['assertion_failures'] > 0]
        results.append({'mutation': name, 'file': str(path.relative_to(ROOT)), 'targets': outcome, 'caught_by': caught})
        (output / 'result.json').write_text(json.dumps({'mutations': results}, indent=2) + '\n')
        broken = [t for t, o in outcome.items() if not o['compiled']]
        missed = 'NOT CAUGHT' + (' (did not compile for ' + ', '.join(broken) + ')' if broken else '')
        print(name + ': ' + ('caught by ' + ', '.join(caught) if caught else missed), flush=True)
    return results


def check_edits():
    return mutation_cases.rows(CASES)


def main(all_cases=CASES, description=__doc__):
    """Runs [all_cases], or those named on the command line; subtype-mutations.py runs its own."""
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument('output', type=Path)
    parser.add_argument('cases', nargs='*', help='case names to run (default: all)')
    args = parser.parse_args()
    unknown = set(args.cases) - {case[0] for case in all_cases}
    assert not unknown, f'unknown cases: {sorted(unknown)}'
    cases = [case for case in all_cases if not args.cases or case[0] in args.cases]
    originals = check(cases)
    output = ROOT / args.output
    output.mkdir(parents=True, exist_ok=False)
    try:
        results = mutate(output, cases, originals)
    finally:
        for path, original in originals.items():
            path.write_bytes(original)
    restored = {}
    for target in sorted({t for case in cases for t in case[4]}):
        status, failures = run(output, 'restored', target)
        restored[target] = {'exit': status, 'assertion_failures': failures}
    missed = [r['mutation'] for r in results if not r['caught_by']]
    report = {'mutations': results, 'missed': missed, 'restored': restored,
              'source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in originals}}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    assert all(r['exit'] == 0 and r['assertion_failures'] == 0 for r in restored.values()), 'restored source fails'
    assert not missed, f'not caught: {missed}'
    print(f'All {len(results)} mutations caught; restored source passes {sorted(restored)}', flush=True)


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
