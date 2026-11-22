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
import time
import xml.etree.ElementTree as ET

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
SHADOW = RENDER / 'shadow_raster.rs'
SESSION = NATIVE / 'render.rs'
RUST, TOUCHES, GLIDE = 'rust:press_bubble', 'rust:press_bubble_touch', 'rust:glide_times'
FRAMES = 'rust:press_bubble_frames'
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
     [FRAMES, TOUCHES]),
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
    ('not-drawn', RENDER / 'raster.rs', '        self.bubble(content)?;\n', '', [RUST, JVM]),
    ('not-styled-after-touch', NATIVE / 'render_input.rs', '        s.style_bubble(false);\n', '', [GLUE, JVM]),
    ('not-restyled-on-update', SESSION, '        session.style_bubble(true);\n', '', [GLUE]),
    ('held-bubble-never-restyled', SESSION, '.filter(|_| again || !styled)', '.filter(|_| !styled)', [GLUE]),
    ('choices-ignored', SESSION, 'let choices = self.semantics.popup_keys.get(key) == Some(&true);',
     'let choices = false;', [GLUE]),
    ('dots-not-restyled', NATIVE / 'render_accessibility.rs', '        s.style_bubble(true);\n', '', [GLUE]),
    ('orientation-not-kept', SESSION, '        s.scene.landscape = landscape;\n', '', [GLUE]),
    ('kept-pixels-not-put-back', FRAME, 'self.pixmap.data_as_u8_slice_mut()[at].copy_from_slice(kept);',
     'let _ = (at, kept);', [TOUCHES]),
    ('release-repaints-the-keys', FRAME, 'let Some(under) = self.under.take() else {',
     'let Some(under) = self.under.take().filter(|_| false) else {', [TOUCHES]),
    ('changes-under-it-not-painted', FRAME, '            if let Some(r) = overlap(changed, under.rect) {\n'
     '                self.paint(scene, r, total)?;\n            }\n', '            let _ = changed;\n', [FRAMES]),
    ('old-bubble-kept-under-the-new', FRAME, 'previous.bubble_damage().is_some_and(|d| meets(d, rect)) || ', '',
     [TOUCHES]),
    ('inner-regions-painted-twice', FRAME, '.any(|(j, &o)| j != i && within(r, o) && (r != o || j < i));',
     '.any(|_| false);', [TOUCHES]),
    ('equal-regions-both-skipped', FRAME, '(r != o || j < i)', 'true', [FRAMES]),
    ('cover-ignored', FRAME, 'scene.bubble_cover().and_then(|c| overlap(r, c));',
     'scene.bubble_cover().and_then(|_| None::<Rect>);', [TOUCHES]),
    ('cover-in-the-top-corners', BUBBLE, 'top: up(r.top + corner + 1.0 + down) - down,',
     'top: up(r.top + 1.0 + down) - down,', [FRAMES]),
    ('cover-in-the-bottom-corners', BUBBLE, 'bottom: ((r.bottom - corner - 1.0 + down)', 'bottom: ((r.bottom - 1.0 + down)',
     [FRAMES]),
    ('cover-off-the-tiles', BUBBLE, '|v: f32| (v / 4.0).ceil() * 4.0)', '|v: f32| v.ceil())', [FRAMES]),
    ('cover-under-glass', BUBBLE, 'if style.background >> 24 != 0xff {', 'if style.background >> 24 == 0 {', [FRAMES]),
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
    return {'rust': rust, 'native': native, 'jvm': jvm}[kind](output, name, target)


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
    main()
