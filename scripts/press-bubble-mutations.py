"""Press-bubble faults: each must trip fresh Rust test panics and/or fresh JVM assertion failures in
every checker it names; source bytes are always restored. python3 -I.

    python3 -I scripts/press-bubble-mutations.py build/press-bubble/mutations [case ...]

Run with no other Gradle or Cargo operation in this checkout. A compilation failure never counts
as caught: Rust needs a "panicked at" line, the JVM an AssertionFailedError in a fresh report.
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
RENDER = ROOT / 'lib/rust/crates/render/src'
NATIVE = ROOT / 'lib/rust/crates/awnative/src'
BUBBLE = RENDER / 'press_bubble.rs'
TOUCH = RENDER / 'keyboard_touch.rs'
SCENE = RENDER / 'scene.rs'
DRAW = RENDER / 'popup_raster.rs'
GESTURES = RENDER / 'gesture_config.rs'
THEMED = RENDER / 'theme_scene.rs'
FRAME = RENDER / 'bubble_frame.rs'
CACHE = RENDER / 'bubble_cache.rs'
SHADOW = RENDER / 'shadow_raster.rs'
SESSION = NATIVE / 'render.rs'
RUST, TOUCHES, GLIDE = 'rust:press_bubble', 'rust:press_bubble_touch', 'rust:glide_times'
FRAMES, CACHED, BLEND = 'rust:press_bubble_frames', 'rust:press_bubble_cache', 'lib:bubble_cache'
GLUE, JVM = 'native:render_bubble_tests', 'jvm:KeyboardRendererTest'
CASES = [
    ('space-shows-one', BUBBLE, 'code > SPACE && code != CJK_SPACE', 'code >= SPACE && code != CJK_SPACE',
     [RUST, TOUCHES]),
    ('cjk-space-shows-one', BUBBLE, 'code > SPACE && code != CJK_SPACE', 'code > SPACE', [RUST]),
    ('advanced-pad-digits-show-one', BUBBLE, '\n        || (mode == "numeric_advanced" && kind == "numeric");',
     ';', [RUST]),
    ('phone-pads-show-one', BUBBLE, 'matches!(mode, "numeric" | "phone" | "phone2")', 'matches!(mode, "numeric")',
     [RUST]),
    ('off-the-right-edge', BUBBLE, 'let left = preview.left.clamp(0.0, room);', 'let left = preview.left.max(0.0);',
     [RUST]),
    ('off-both-edges', BUBBLE, 'let left = preview.left.clamp(0.0, room);', 'let left = preview.left;', [RUST]),
    ('portrait-in-landscape', BUBBLE, 'self.desired_key, self.landscape)', 'self.desired_key, false)', [RUST, GLUE]),
    ('drawn-under-the-popup', BUBBLE, 'self.bubble.as_ref().filter(|_| self.popup.is_none())?', 'self.bubble.as_ref()?',
     [RUST, TOUCHES]),
    ('drawn-for-any-key', BUBBLE, '        if self.bubbles.get(bubble.key) != Some(&true) {\n'
     '            return None;\n        }\n', '', [RUST]),
    ('shadow-not-repainted', BUBBLE, 'Some(crate::shadow_raster::extent(r, style.shadow))', 'Some(r)',
     [FRAMES, CACHED]),
    ('shadow-extent-short', SHADOW, '(2.5 * blur * shadow.max(0.0), offset', '(1.0 * blur * shadow.max(0.0), offset',
     [FRAMES]),
    ('bubble-not-repainted', SCENE, '            rects.extend(self.bubble_damage());\n'
     '            rects.extend(next.bubble_damage());\n', '', [TOUCHES]),
    ('old-bubble-left-behind', SCENE, '            rects.extend(self.bubble_damage());\n', '', [TOUCHES]),
    ('setting-ignored', TOUCH, '.find(|&k| on && scene.bubbles', '.find(|&k| scene.bubbles', [TOUCHES, JVM]),
    ('slide-leaves-it', TOUCH, 'if scene.bubble.as_ref().map(|b| b.key) != key {', 'if scene.bubble.is_none() {',
     [TOUCHES]),
    ('cancel-leaves-it', TOUCH, '        scene.bubble = None;\n        self.pointers.clear();',
     '        self.pointers.clear();', [TOUCHES]),
    ('glide-leaves-it', TOUCH, '        scene.bubble = None;\n        Self::record(', '        Self::record(', [TOUCHES]),
    ('setting-not-decoded', GESTURES, 'bubble: values.get(11)', 'bubble: values.get(10)', [RUST, JVM]),
    ('thirteen-values-accepted', GESTURES, '!(9..=12).contains', '!(9..=13).contains', [RUST, GLIDE, JVM]),
    ('label-fills-the-bubble', DRAW, 'bottom: (inner.top + key.bounds.visible.height().round()).min(inner.bottom),',
     'bottom: inner.bottom,', [RUST]),
    ('dots-not-drawn', DRAW, '            self.icon(MORE, dots, at)?;\n', '            let _ = (MORE, dots, at);\n',
     [RUST]),
    ('dots-without-choices', THEMED, 'choices.then(|| self.style(&request.element("key-popup-extended-indicator", 0)))',
     'Some(self.style(&request.element("key-popup-extended-indicator", 0)))', [RUST, GLUE]),
    ('dots-unstyled-by-the-box', RENDER / 'theme.rs',
     '"key-popup-element" | "key-popup-extended-indicator" => Some("key-popup-box"),',
     '"key-popup-element" => Some("key-popup-box"),', [RUST]),
    ('keys-not-marked', THEMED, '        crate::press_bubble::mark(scene, mode, types);\n', '', [GLUE, JVM]),
    ('not-drawn', CACHE, '        self.bubble(scene)?;\n        let mut bytes', '        let mut bytes', [RUST, JVM]),
    ('not-laid-in-a-whole-draw', RENDER / 'raster.rs', '            self.blend_bubble(scene, damage, &mut total)?;\n',
     '', [RUST, FRAMES]),
    ('not-styled-after-touch', NATIVE / 'render_input.rs', '        s.style_bubble(false);\n', '', [GLUE, JVM]),
    ('not-restyled-on-update', SESSION, '        session.style_bubble(true);\n', '', [GLUE]),
    ('held-bubble-never-restyled', SESSION, '.filter(|_| again || !styled)', '.filter(|_| !styled)', [GLUE]),
    ('choices-ignored', SESSION, 'let choices = self.semantics.popup_keys.get(key) == Some(&true);',
     'let choices = false;', [GLUE]),
    ('dots-not-restyled', NATIVE / 'render_accessibility.rs', '        s.style_bubble(true);\n', '', [GLUE]),
    ('orientation-not-kept', SESSION, '        s.scene.landscape = landscape;\n', '', [GLUE]),
    ('kept-pixels-not-put-back', FRAME, 'copy(buffer, frame, &under.bytes, under.pixels, under.pixels);',
     'let _ = (buffer, frame);', [TOUCHES, FRAMES]),
    ('new-bubble-not-kept', FRAME, 'self.under = Some(self.keep(scene, rect));', 'let _ = self.keep(scene, rect);',
     [TOUCHES]),
    ('not-laid-in-a-frame', FRAME, '            self.blend_bubble(scene, rect, &mut total)?;\n', '',
     [TOUCHES, FRAMES, CACHED]),
    ('held-bubble-put-back-each-frame', FRAME, 'Some(under) if same => self.under = Some(under),',
     'Some(under) if same && false => self.under = Some(under),', [FRAMES]),
    ('kept-across-a-relayout', FRAME, '.filter(|u| !restructured && Some(u.rect) == old)',
     '.filter(|u| Some(u.rect) == old)', [CACHED]),
    ('gone-bubble-left-without-kept-pixels', FRAME, 'None => paint.extend(old),', 'None => {}', [FRAMES]),
    ('whole-bubble-damage-repainted', FRAME, 'changed_under(previous, scene, regions).to_vec()', 'regions.to_vec()',
     [TOUCHES]),
    ('repaint-under-it-not-kept', FRAME, 'copy(&mut under.bytes, under.pixels, pixels, frame, fresh);',
     'let _ = (pixels, frame, fresh);', [FRAMES]),
    ('repaint-under-it-hides-the-bubble', FRAME, '        self.blend_bubble(scene, r, total)\n', '        Ok(())\n',
     [FRAMES]),
    ('inner-regions-painted-twice', FRAME, '.any(|(j, &o)| j != i && within(r, o) && (r != o || j < i));',
     '.any(|_| false);', [CACHED]),
    ('equal-regions-both-skipped', FRAME, '(r != o || j < i)', 'true', [FRAMES]),
    ('images-never-dropped', FRAME, '            self.images.clear();\n', '', [CACHED]),
    ('label-not-in-the-look', CACHE, 'label: key.label.clone(),', 'label: String::new(),', [CACHED]),
    ('key-height-not-in-the-look', CACHE, 'key_height: key.bounds.visible.height(),', 'key_height: 0.0,', [CACHED]),
    ('bounds-not-in-the-look', CACHE, '        bounds,\n        style: style.clone(),',
     '        bounds: Rect::default(),\n        style: style.clone(),', [CACHED]),
    ('style-not-in-the-look', CACHE, 'style: style.clone(),', 'style: Style::default(),', [CACHED]),
    ('dots-not-in-the-look', CACHE, 'indicator: bubble.indicator.clone(),', 'indicator: None,', [CACHED]),
    ('theme-change-keeps-them', CACHE, 'a.background != b.background\n        || ', '', [CACHED]),
    ('more-keys-keep-them', CACHE, '\n        || a.keys.len() != b.keys.len()', '', [CACHED]),
    ('new-layout-keeps-them', CACHE, '\n        || a.keys\n            .iter()\n            .zip(&b.keys)\n'
     '            .any(|(x, y)| x.bounds != y.bounds)', '', [CACHED]),
    ('shift-drops-them', CACHE, 'x.bounds != y.bounds', 'x != y', [CACHED]),
    ('unbounded', CACHE, 'if kept.len() >= KEPT {', 'if false {', [CACHED]),
    ('used-image-not-kept-longest', CACHE, 'kept[i..].rotate_left(1);', 'let _ = i;', [CACHED]),
    ('shadow-copied-over', CACHE, 'if run.solid {', 'if true {', [CACHED]),
    ('faint-pixels-dropped', CACHE, 'if pixel[3] == 0 {', 'if pixel[3] < 255 {', [CACHED, BLEND]),
    ('blend-rounds-down', CACHE, 'let x = u16::from(*to) * keep + 128;', 'let x = u16::from(*to) * keep;', [BLEND]),
]


def env():
    e = dict(os.environ)
    e.setdefault('CARGO_BUILD_JOBS', '6')
    return e


def cargo(output, name, arguments, label):
    with (output / (name + '.' + label + '.log')).open('wb') as log:
        result = subprocess.run(['cargo', 'test', '-q', *arguments], cwd=ROOT / 'lib/rust', stdout=log,
                                stderr=subprocess.STDOUT, env=env())
    text = (output / (name + '.' + label + '.log')).read_text(errors='replace')
    return result.returncode, sum('panicked at' in line for line in text.splitlines())


def rust(output, name, target):
    return cargo(output, name, ['-p', 'render', '--test', target], target)


def native(output, name, module):
    return cargo(output, name, ['-p', 'awnative', '--lib', module], module)


def lib(output, name, module):
    return cargo(output, name, ['-p', 'render', '--lib', module], 'lib-' + module)


def jvm(output, name, test):
    started = time.time_ns()
    with (output / (name + '.' + test + '.log')).open('wb') as log:
        result = subprocess.run(['./gradlew', '--max-workers=6', ':lib:rust:testDebugUnitTest', '--tests', '*' + test],
                                cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=env())
    failures = 0
    for path in (ROOT / 'lib/rust/build/test-results/testDebugUnitTest').glob('TEST-*' + test + '.xml'):
        if path.stat().st_mtime_ns >= started:
            failures += sum('AssertionFailedError' in f.attrib.get('type', '')
                            for f in ET.parse(path).getroot().iter('failure'))
    return result.returncode, failures


def check(output, name, checker):
    kind, target = checker.split(':')
    return {'rust': rust, 'native': native, 'lib': lib, 'jvm': jvm}[kind](output, name, target)


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
        for name, path, old, new, checkers in cases:
            source = originals[path].decode()
            assert source.count(old) == 1, 'mutation location changed: ' + name
            path.write_text(source.replace(old, new))
            try:
                caught = {}
                for checker in checkers:
                    status, failures = check(output, name, checker)
                    assert status != 0 and failures, f'{name} must fail fresh assertions in {checker}'
                    caught[checker] = failures
                results.append({'mutation': name, 'caught': caught})
            finally:
                path.write_bytes(originals[path])
            print(name + ': caught by ' + ', '.join(f'{c} ({n})' for c, n in caught.items()), flush=True)
    finally:
        for path, original in originals.items():
            path.write_bytes(original)
    restored = {}
    for checker in sorted({c for case in cases for c in case[4]}):
        status, failures = check(output, 'restored', checker)
        assert status == 0 and not failures, 'restored source fails ' + checker
        restored[checker] = status
    report = {'mutations': results, 'restored_exit': restored,
              'source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in originals}}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'All {len(results)} mutations caught; restored source passes {sorted(restored)}', flush=True)


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
