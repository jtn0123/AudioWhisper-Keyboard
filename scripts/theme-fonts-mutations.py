"""Theme-font faults: each must trip fresh test failures in every target it names; source bytes are
always restored. python3 -I.

    python3 -I scripts/theme-fonts-mutations.py build/theme-fonts/mutations [case ...]

A Rust target is a test binary of the render crate, or `awnative` for that crate's library tests
of the session; `kotlin` is the JVM test `RendererThemeFontsTest`. Run with no other Cargo or
Gradle operation in this checkout, and do not read the source as the truth while it runs: it is
mutated then. A compilation failure never counts as caught: a Rust case needs a "panicked at" line
and a Kotlin one a failed test in its JUnit report.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import xml.etree.ElementTree as ElementTree

sys.dont_write_bytecode = True  # importing mutation_cases would otherwise leave a __pycache__ in scripts/
sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutation_cases

ROOT = Path(__file__).resolve().parent.parent
RENDER = ROOT / 'lib/rust/crates/render/src'
FONTS = RENDER / 'theme_fonts.rs'
TEXT = RENDER / 'text.rs'
THEME = RENDER / 'theme.rs'
NATIVE = ROOT / 'lib/rust/crates/awnative/src'
KOTLIN = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/render/RendererThemeFonts.kt'
DRAWN, AW, KT = 'theme_fonts', 'awnative', 'kotlin'
NAMED = 'FontFamilyName::Named(alias.to_string().into()),\n'
SANS = '                        FontFamilyName::Generic(GenericFamily::SansSerif),\n'
CASES = [
    # The theme gives elements its fonts, and forgets styles resolved before they loaded.
    ('theme-font-ignored', THEME, 'map_or(style.font, |(_, font)| *font)', 'map_or(style.font, |_| style.font)',
     [DRAWN]),
    ('styles-kept-across-loads', THEME, 'files);\n        self.cache.clear();\n', 'files);\n', [DRAWN]),
    # The text engine draws a theme font first and today's chain after it.
    ('text-ignores-theme-font', TEXT, 'match self.theme_alias(id) {', 'match self.theme_alias(id).filter(|_| false) {',
     [DRAWN, AW]),
    ('no-default-chain-after', TEXT, SANS, '', [DRAWN]),
    ('default-chain-first', TEXT, NAMED + SANS, SANS.lstrip() + '                        ' + NAMED, [DRAWN]),
    ('registration-misread', TEXT, 'registered |= !families.is_empty();', 'registered |= families.is_empty();',
     [DRAWN]),
    ('one-alias-for-every-font', TEXT, 'format!("audiowhisper theme font {id}")',
     '"audiowhisper theme font".to_string()', [DRAWN]),
    ('declared-weight-ignored', TEXT, 'weight: Some(parley::FontWeight::new(face.weight as f32)),', 'weight: None,',
     [DRAWN]),
    ('declared-style-ignored', TEXT, 'style: Some(if face.italic {', 'style: Some(if false {', [DRAWN]),
    # Reading the `@font` sets.
    ('weight-dropped', FONTS, 'Some(Value::FontWeight(w)) => *w,', 'Some(Value::FontWeight(_)) => 400,', [DRAWN]),
    ('italic-dropped', FONTS, 'if *s == text("italic")),', 'if *s == text("oblique")),', [DRAWN]),
    ('any-uri-matches', FONTS, 'files.iter().find(|(u, _)| *u == uri)?', 'files.first()?', [DRAWN]),
    ('rules-not-merged', FONTS, 'Some((_, known)) => known.extend(faces),',
     'Some(_) => named.push((name.clone(), faces.collect())),', [DRAWN]),
    ('empty-rule-kept', FONTS, '        .filter(|(_, faces)| !faces.is_empty())\n', '', [DRAWN]),
    # One id per distinct font, in every text context.
    ('same-files-new-font', FONTS, 'if let Some(id) = all.iter().position(|f| **f == *faces) {',
     'if let Some(id) = None::<usize> {', [DRAWN]),
    ('files-not-compared', FONTS, '\n            && self.blob.data() == other.blob.data()', '', [DRAWN]),
    ('registry-too-small', FONTS, 'if all.len() >= MAX_FAMILIES {', 'if !all.is_empty() {', [DRAWN]),
    # A broken file is no font, and never a crash.
    ('not-checked-whole', FONTS, 'whole(&bytes).then(|| FontBlob::from(bytes))', 'Some(FontBlob::from(bytes))',
     [DRAWN]),
    ('crash-on-missing', FONTS, 'std::fs::metadata(path).ok()?', 'std::fs::metadata(path).unwrap()', [DRAWN, AW]),
    ('crash-on-unreadable', FONTS, 'std::fs::read(path).ok()?', 'std::fs::read(path).unwrap()', [DRAWN]),
    ('size-unchecked', FONTS, 'if size > MAX_FILE {', 'if false && size > MAX_FILE {', [DRAWN]),
    ('tables-past-the-end', FONTS, '.is_some_and(|end| end <= bytes.len())', '.is_some()', [DRAWN]),
    ('cmap-unchecked', FONTS, '\n                && font.cmap().is_ok()', '', [DRAWN]),
    # The session takes Kotlin's files.
    ('lone-uri-accepted', NATIVE / 'render_theme.rs', 'if !flat.len().is_multiple_of(2) {', 'if false {', [AW]),
    ('files-not-loaded', NATIVE / 'render.rs', 'theme.load_fonts(&crate::render_theme::font_files(&texts(fonts))?);',
     'crate::render_theme::font_files(&texts(fonts))?;', [AW]),
    # Kotlin lists each resolved `src` as its URI then its path.
    ('unresolved-kept', KOTLIN, 'resolver.resolveAbsolutePath(uri).getOrNull()?.let { listOf(uri, it) }.orEmpty()',
     'listOf(uri, resolver.resolveAbsolutePath(uri).getOrNull().orEmpty())', [KT]),
    ('path-before-uri', KOTLIN, 'let { listOf(uri, it) }', 'let { listOf(it, uri) }', [KT]),
]


def check_edits():
    return mutation_cases.rows(CASES)


def env():
    e = dict(os.environ)
    e.setdefault('CARGO_BUILD_JOBS', '6')
    e.setdefault('ANDROID_HOME', str(Path.home() / 'Library/Android/sdk'))
    return e


def kotlin(log_path):
    reports = ROOT / 'app/build/test-results/testDebugUnitTest'
    for old in reports.glob('*RendererThemeFontsTest*.xml'):
        old.unlink()
    with log_path.open('wb') as log:
        result = subprocess.run(['./gradlew', '-q', ':app:testDebugUnitTest', '--tests',
                                 'com.jtn0123.audiowhisper.render.RendererThemeFontsTest'],
                                cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=env())
    failures = 0
    for report in reports.glob('*RendererThemeFontsTest*.xml'):
        suite = ElementTree.parse(report).getroot()
        failures += int(suite.get('failures', 0)) + int(suite.get('errors', 0))
    return result.returncode, failures


def run(output, name, target):
    log_path = output / (name + '.' + target + '.log')
    if target == KT:
        return kotlin(log_path)
    command = ['cargo', 'test', '-q', '-p', 'awnative', '--lib', 'render_theme'] if target == AW else \
        ['cargo', 'test', '-q', '-p', 'render', '--test', target]
    with log_path.open('wb') as log:
        result = subprocess.run(command, cwd=ROOT / 'lib/rust', stdout=log, stderr=subprocess.STDOUT, env=env())
    text = log_path.read_text(errors='replace')
    return result.returncode, sum('panicked at' in line for line in text.splitlines())


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
    originals = {path: path.read_bytes() for _, path, _, _, _ in cases}
    results = []
    try:
        for name, path, old, new, targets in cases:
            source = originals[path].decode()
            assert source.count(old) == 1, 'mutation location changed: ' + name
            path.write_text(source.replace(old, new))
            try:
                caught = {}
                for target in targets:
                    status, failures = run(output, name, target)
                    assert status != 0 and failures, f'{name} must fail fresh assertions in {target}'
                    caught[target] = failures
                results.append({'mutation': name, 'caught': caught})
            finally:
                path.write_bytes(originals[path])
            print(name + ': caught by ' + ', '.join(f'{t} ({n})' for t, n in caught.items()), flush=True)
    finally:
        for path, original in originals.items():
            path.write_bytes(original)
    restored = {}
    for target in sorted({t for case in cases for t in case[4]}):
        status, failures = run(output, 'restored', target)
        assert status == 0 and not failures, 'restored source fails ' + target
        restored[target] = status
    report = {'mutations': results, 'restored_exit': restored,
              'source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in originals}}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'All {len(results)} mutations caught; restored source passes {sorted(restored)}', flush=True)


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
