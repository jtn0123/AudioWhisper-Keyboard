"""Key-label fit faults: each must trip fresh Rust test panics in every test target it names; source
bytes are always restored. python3 -I.

    python3 -I scripts/key-label-fit-mutations.py build/key-label-fit/mutations [case ...]

Run with no other Cargo operation in this checkout, and do not read the source as the truth while
it runs: it is mutated then. A compilation failure never counts as caught: a case needs a
"panicked at" line.
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
RENDER = ROOT / 'lib/rust/crates/render/src'
FIT = RENDER / 'label_fit.rs'
RASTER = RENDER / 'raster.rs'
KEYS = RENDER / 'raster_keys.rs'
POPUP = RENDER / 'popup_raster.rs'
SHARED = RENDER / 'smartbar_fit.rs'
TEXT = RENDER / 'text.rs'
KEY, SCRIPTS = 'key_label_fit', 'key_label_fit_scripts'
CASES = [
    # The check that keeps a label's old drawing only while it stays inside its box.
    ('never-checked', FIT, 'at, Some((room, lines)))?', 'at, None)?', [KEY, SCRIPTS]),
    ('left-unchecked', RASTER, '(x < room.left - SLACK\n                || ', '(', [KEY]),
    ('right-unchecked', RASTER, '\n                || x + width > room.right + SLACK', '', [KEY]),
    ('top-unchecked', RASTER, '\n                || y < room.top - SLACK', '', [KEY]),
    ('bottom-unchecked', RASTER, '\n                || y + height > room.bottom + SLACK', '', [KEY]),
    ('lines-unchecked', RASTER, '\n                || layout.len() > lines)', ')', [KEY]),
    # Drawing a fitted label.
    ('not-clipped', FIT, '        self.context.push_clip_rect(&crate::raster::clip(bounds));\n', '', [KEY]),
    ('clipped-to-the-key', FIT, 'if fitted.spills { place.outer } else { room }', 'place.outer', [KEY]),
    ('visible-unbounded', FIT, 'if fitted.spills { place.outer } else { room }',
     'if fitted.spills { Rect { left: -1e4, top: -1e4, right: 1e4, bottom: 1e4 } } else { room }', [KEY]),
    ('centred-when-hung', FIT, '                _ if fitted.hangs => 0.0,\n', '', [KEY]),
    # The fit kept, and made again when what it fits changes.
    ('not-kept', FIT, '        self.fits.kept.insert(key, fitted.clone());\n', '', [KEY]),
    ('kept-too-few', FIT, 'self.fits.kept.len() >= 256', 'self.fits.kept.len() >= 64', [KEY]),
    ('kept-across-sizes', FIT, '[style.size, room.width(), room.height()]', '[0.0, room.width(), room.height()]',
     [KEY]),
    ('kept-across-keys', FIT, '[style.size, room.width(), room.height()]', '[style.size, 0.0, 0.0]', [KEY]),
    ('kept-across-themes', FIT, '            style.weight,\n            style.rules,\n',
     '            style.weight,\n            TextRules::default(),\n', [KEY]),
    # Compose's rules.
    ('max-lines-ignored', FIT, 'let mut keep = ranges.len().min(max);', 'let mut keep = ranges.len();', [KEY]),
    ('height-ignored', FIT, 'if ellipsis && keep > 1 {', 'if false && keep > 1 {', [KEY]),
    ('no-ellipsis', FIT, 'if keep < ranges.len() && ellipsis {', 'if false && ellipsis {', [KEY, SCRIPTS]),
    ('paragraph-unbounded', FIT, 'text.measure(label, size, mono, weight).min(width)',
     'text.measure(label, size, mono, weight)', [KEY]),
    ('start-always-left', FIT, 'let start = if rtl { free } else { 0.0 };', 'let start = if rtl { 0.0 } else { 0.0 };',
     [SCRIPTS]),
    ('centre-ignored', FIT, 'Align::Center => free / 2.0,', 'Align::Center => 0.0,', [KEY]),
    ('lines-keep-spaces', TEXT, 'text[range.clone()].trim_end().len()', 'text[range.clone()].len()', [KEY]),
    # The default where the theme says nothing.
    ('never-shrunk-to-height', FIT, '    fit.scale = smartbar_fit::text_scale(fit.scale, tall, height);\n', '', [KEY]),
    ('never-ellipsised', FIT, 'let line = if measure(label) <= width {', 'let line = if true {', [KEY]),
    ('ellipsis-dropped', SHARED, 'format!("{}…", &text[..ends[low - 1]])', 'text[..ends[low - 1]].to_owned()',
     [KEY, SCRIPTS]),
    # Where each label is fitted.
    ('padding-ignored', KEYS, 'at, inset(r, style.padding), place)', 'at, r, place)', [KEY]),
    ('phone-digit-centred', KEYS, 'self.text_at(&key.label, style, r, telpad.then_some(false))',
     'self.text_at(&key.label, style, r, None)', [KEY]),
    ('hint-centred', KEYS, 'let bias = if telpad { [0.75, 0.5] } else { [1.0, 0.0] };', 'let bias = [0.5, 0.5];',
     [KEY]),
    ('hint-kept-to-its-corner', KEYS, 'self.contained_label(&key.hint, hint, at, r, place)',
     'self.contained_label(&key.hint, hint, at, at, place)', [KEY]),
    ('choice-unfitted', POPUP, 'self.contained_label(label, style, r, r, Place::centred(r))?;',
     'self.label(label, style.size, style.foreground, r, style.mono, style.weight)?;', [KEY]),
    ('bubble-unfitted', POPUP, 'self.contained_label(&key.label, style, label, label, Place::centred(r))?;',
     'self.squeezed_label(&key.label, (style.size, style.mono, style.weight), 1.0, style.foreground, label)?;',
     [SCRIPTS]),
]


def env():
    e = dict(os.environ)
    e.setdefault('CARGO_BUILD_JOBS', '6')
    return e


def rust(output, name, target):
    log_path = output / (name + '.' + target + '.log')
    with log_path.open('wb') as log:
        result = subprocess.run(['cargo', 'test', '-q', '-p', 'render', '--test', target], cwd=ROOT / 'lib/rust',
                                stdout=log, stderr=subprocess.STDOUT, env=env())
    text = log_path.read_text(errors='replace')
    return result.returncode, sum('panicked at' in line for line in text.splitlines())


def check_edits():
    return mutation_cases.rows(CASES)


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
                    status, failures = rust(output, name, target)
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
        status, failures = rust(output, 'restored', target)
        assert status == 0 and not failures, 'restored source fails ' + target
        restored[target] = status
    report = {'mutations': results, 'restored_exit': restored,
              'source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in originals}}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'All {len(results)} mutations caught; restored source passes {sorted(restored)}', flush=True)


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
