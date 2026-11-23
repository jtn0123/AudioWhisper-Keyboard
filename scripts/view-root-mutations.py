"""Plain-View root faults (phase 3 item 4): each must trip a fresh assertion in at least one of the
test targets it names; source bytes are always restored. python3 -I.

    python3 -I scripts/view-root-mutations.py build/view-root/mutations [case ...]
    python3 -I scripts/view-root-mutations.py --check     # only that every location is still there

The runner is theme-subtype-mutations.py's, with the same rules: run with no other Cargo or Gradle
operation in this checkout, a compilation failure never counts as caught, and every target a case
names is run and recorded. The cases are the decisions the root asks Rust for: the window plan
(`render::ime_root`) and the window's background (`imetheme::window` and its row). The Kotlin that
sends the facts is checked on a device by view-root-diff.py, which no mutation run drives.
"""
import importlib.util
import sys
from pathlib import Path

# Loading the runner and mutation_cases as modules would otherwise leave a __pycache__ in scripts/.
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutation_cases

SPEC = importlib.util.spec_from_file_location('theme_subtype_mutations', Path(__file__).parent / 'theme-subtype-mutations.py')
RUNNER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUNNER)

ROOT = RUNNER.ROOT
PLAN = ROOT / 'lib/rust/crates/render/src/ime_root.rs'
THEME = ROOT / 'lib/rust/crates/imetheme/src'
WINDOW, CALLS = THEME / 'window.rs', THEME / 'calls.rs'
# The targets: the plan's and the theme's Rust tests, and the search bar's plan across JNI.
UNIT, THEMES, SEARCH = 'cargo:ime_root', 'cargo:imetheme', 'jvm:search'
BOTH = [UNIT, SEARCH]
CASES = [
    # Compose's arithmetic.
    ('halves-round-down', PLAN, '(f64::from(x) + 0.5).floor() as i32', '(f64::from(x) + 0.49).floor() as i32', [UNIT]),
    ('rounding-remainder-dropped', PLAN, '(share(*w) + step).max(0) as i32', 'share(*w).max(0) as i32', [UNIT]),
    ('base-height-ignored', PLAN, '((min + max) / 2.0).max(a[f::BASE_HEIGHT]) * 0.21', '((min + max) / 2.0) * 0.21', BOTH),
    ('fewer-than-four-rows', PLAN, '        .max(4)\n', '        .max(3)\n', [UNIT]),
    # The panel showing.
    ('overflow-over-editor', PLAN, '(true, _, _) => 3.0,\n        (_, true, _) => 2.0,',
     '(_, true, _) => 2.0,\n        (true, _, _) => 3.0,', [UNIT]),
    ('gifs-as-emoji', PLAN, '(_, _, 1) if flag(f::GIFS) => 5.0,', '(_, _, 1) if flag(f::GIFS) => 1.0,', [UNIT]),
    ('search-bar-not-planned', PLAN, '    if search {\n        gif_search(', '    if false {\n        gif_search(', BOTH),
    # The row: safe insets, one-handed panels, the bottom offset.
    ('safe-left-ignored', PLAN, 'let (left, top) = (int(f::SAFE_LEFT), int(f::SAFE_TOP));',
     'let (left, top) = (0, int(f::SAFE_TOP));', [UNIT]),
    ('safe-right-ignored', PLAN, 'let content = int(f::WIDTH) - left - int(f::SAFE_RIGHT);',
     'let content = int(f::WIDTH) - left;', [UNIT]),
    ('safe-bottom-ignored', PLAN, 'round(offset as f32 * d) + int(f::SAFE_BOTTOM);', 'round(offset as f32 * d);', [UNIT]),
    ('offset-ignored', PLAN, 'let box_height = top + row + round(offset as f32 * d)', 'let box_height = top + row', [UNIT]),
    ('offset-of-the-other-orientation', PLAN, 'f::OFFSET_LANDSCAPE\n    } else {\n        f::OFFSET_PORTRAIT',
     'f::OFFSET_PORTRAIT\n    } else {\n        f::OFFSET_LANDSCAPE', [UNIT]),
    ('panel-on-the-mode-side', PLAN, 'let (start_panel, end_panel) = (w[1] > 0.0, w[2] > 0.0);',
     'let (start_panel, end_panel) = (w[2] > 0.0, w[1] > 0.0);', [UNIT]),
    ('panel-as-tall-as-keys', PLAN, '            row = row.max(panel_height);\n', '', [UNIT]),
    # The column under a full-screen extract view, and the surface's rise above the window.
    ('spacer-under-extract', PLAN, 'let spacer = !(a[f::FULLSCREEN] != 0.0 && a[f::EXTRACT_SHOWN] != 0.0);',
     'let spacer = !(a[f::FULLSCREEN] != 0.0);', [UNIT]),
    ('at-most-not-filled', PLAN, '1 if spacer => spec,', '1 if spacer => box_height,', [UNIT]),
    ('box-at-the-top', PLAN, '        (view - box_height).max(0)\n', '        0\n', [UNIT]),
    ('rise-always', PLAN, 'out[p::RISE] = if spacer { 0.0 } else { 1.0 };', 'out[p::RISE] = 1.0;', [UNIT]),
    ('rise-never', PLAN, 'out[p::RISE] = if spacer { 0.0 } else { 1.0 };', 'out[p::RISE] = 0.0;', [UNIT]),
    ('editor-surface-keyboard-tall', PLAN, 'let dp = if out[p::PANEL] == 3.0 {', 'let dp = if false {', [UNIT]),
    # The window's background.
    ('white-default', WINDOW, 'background: color.map(argb).unwrap_or(BACKGROUND),',
     'background: color.map(argb).unwrap_or(0xFFFF_FFFF),', [THEMES]),
    ('light-bars-over-an-image', WINDOW, 'light_bars: uri.is_none() && color', 'light_bars: color', [THEMES]),
    ('contrast-never-enforced', WINDOW, 'contrast: uri.is_some(),', 'contrast: false,', [THEMES]),
    ('window-outside-root', WINDOW, '&["root", "window"]', '&["window"]', [THEMES]),
    ('scale-misnamed', WINDOW, '.position(|s| text(s) == *name)', '.position(|s| text(s) != *name)', [THEMES]),
    ('escape-above-the-theme', WINDOW, '                parts.pop()?;', '                parts.pop();', [THEMES]),
    ('percent-kept', WINDOW, "        if b == b'%' {", '        if false {', [THEMES]),
    ('query-kept', WINDOW, "    path = &path[..path.find(['?', '#']).unwrap_or(path.len())];\n", '', [THEMES]),
    ('crop-fits', WINDOW, '_ => (sx.max(sy), sx.max(sy)),', '_ => (fit, fit),', [THEMES]),
    ('inside-scales-up', WINDOW,
     '"inside" if image[0] <= canvas[0] && image[1] <= canvas[1] => (1.0, 1.0),\n', '', [THEMES]),
    ('image-at-the-left', WINDOW, 'let left = canvas[0] / 2.0 - size[0] / 2.0;', 'let left = 0.0_f32;', [THEMES]),
    # The row that crosses from Android and back.
    ('image-size-swapped', CALLS, 'size(window::IMAGE_WIDTH, window::IMAGE_HEIGHT)?', 'size(window::IMAGE_HEIGHT, window::IMAGE_WIDTH)?',
     [THEMES]),
    ('unknown-size-placed', CALLS, '.all(|v| v.is_finite() && *v > 0.0);', '.all(|v| v.is_finite());', [THEMES]),
    ('light-bars-for-contrast', CALLS, 'wire::flag::encode(found.light_bars)', 'wire::flag::encode(found.contrast)', [THEMES]),
]
RUNNER.CARGO.update({UNIT: ['-p', 'render', '--lib', 'ime_root'], THEMES: ['-p', 'imetheme']})
RUNNER.JVM[SEARCH] = (':app:testDebugUnitTest', ['*RendererSearchTransportTest'],
                      ROOT / 'app/build/test-results/testDebugUnitTest')


def check_edits():
    return mutation_cases.rows(CASES)


if __name__ == '__main__':
    mutation_cases.check_flag()
    RUNNER.main(CASES, __doc__)
