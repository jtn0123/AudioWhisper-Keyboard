//! The plain-View root's layout (`wire::ime_window`): where Compose's root put the window box,
//! the one-handed panels, the keyboard, its surface and the inline autofill chips.
//!
//! Compose's root was `ImeUiWrapper`'s column (a spacer over the box unless a full-screen window
//! shows its extract view), `ImeUi`'s box and row (safe drawing padding and the bottom offset
//! around weighted panels and keyboard) and `RustKeyboardLayout`'s box. Each size is worked here
//! in Compose's own arithmetic: dp as `f32`, `roundToPx` rounding halves up, and a row's weighted
//! children sized as `RowColumnMeasurePolicy` sizes them, so the two roots agree to the pixel.

use wire::ime_window::{facts as f, plan as p};

use crate::ime_chrome::{one_handed, row_heights};
use crate::smartbar::{self, Config};

/// `Math.round` and Compose's `roundToPx`: the nearest whole number, halves up.
pub fn round(x: f32) -> i32 {
    (f64::from(x) + 0.5).floor() as i32
}

/// Sizes along a row of weighted children sharing `space`, as Compose's row gives them: each
/// its rounded share, and the pixels rounding lost or gained handed out one each from the start.
pub fn weighted(space: i32, weights: &[f32]) -> Vec<i32> {
    let space = i64::from(space.max(0));
    let total = weights.iter().fold(0.0f32, |sum, w| sum + w);
    let unit = space as f32 / total;
    let share = |w: f32| i64::from(round(unit * w));
    let mut remainder = space - weights.iter().map(|w| share(*w)).sum::<i64>();
    let sized = weights.iter().map(|w| {
        let step = remainder.signum();
        remainder -= step;
        (share(*w) + step).max(0) as i32
    });
    sized.collect()
}

/// `FlorisImeSizing.calcInputViewHeight`: the height rows are a fraction of.
fn input_view_height(a: &[f32], bars: i32, landscape: bool) -> f32 {
    let height = a[f::DISPLAY_HEIGHT] as i32 - bars;
    let width = a[f::DISPLAY_WIDTH] as i32;
    let min = a[f::MIN_FRACTION] * (if landscape { height } else { width }) as f32;
    let max = a[f::MAX_FRACTION] * height as f32;
    ((min + max) / 2.0).max(a[f::BASE_HEIGHT]) * 0.21
}

/// The panel showing, as `RendererPanels.kind` numbers it.
fn panel(a: &[f32]) -> f32 {
    let flag = |i: usize| a[i] != 0.0;
    match (flag(f::EDITOR), flag(f::OVERFLOW), a[f::UI_MODE] as i32) {
        (true, _, _) => 3.0,
        (_, true, _) => 2.0,
        (_, _, 2) => 4.0,
        (_, _, 1) if flag(f::GIFS) => 5.0,
        (_, _, 1) => 1.0,
        _ => 0.0,
    }
}

/// `gifSearchBarConfig`: the Smartbar while a GIF search is typed, one row of the search bar.
fn gif_search(config: &mut [f32], row: f32) {
    config[0] = 1.0;
    config[1] = 0.0;
    config[2..=7].fill(0.0);
    config[9] = 0.0;
    config[11] = row;
    config[12..].fill(0.0);
}

/// The plan for `facts` (`wire::ime_window::facts`, then the Smartbar config): the `plan` fields,
/// then the Smartbar config the keyboard draws. None if a fact is missing or not a number.
pub fn layout(a: &[f32]) -> Option<Vec<f32>> {
    let config = a.get(f::SIZE..)?;
    if !matches!(config.len(), 12 | 14 | 17) || a.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let (density, system) = (a[f::DENSITY], a[f::SYSTEM_DENSITY]);
    if density <= 0.0 || system <= 0.0 {
        return None;
    }
    let flag = |i: usize| a[i] != 0.0;
    let int = |i: usize| a[i] as i32;
    let landscape = flag(f::LANDSCAPE);
    let bars = int(f::NAVIGATION_BAR) + int(f::STATUS_BAR);
    let base = input_view_height(a, bars, landscape);
    let [row_px, bar_px] = row_heights(
        base,
        int(f::HEIGHT_PORTRAIT),
        int(f::HEIGHT_LANDSCAPE),
        int(f::ONE_HANDED_SCALE),
        flag(f::ONE_HANDED),
        landscape,
    );
    // ProvideKeyboardRowBaseHeight's dp sizes, from the system's density as ViewUtils.px2dp.
    let (row_dp, bar_dp) = (row_px / system, bar_px / system);
    let mut sized = Sized {
        density,
        keyboard_dp: row_dp * rows(a) as f32,
        config: config.to_vec(),
    };
    sized.config[10] = density;
    sized.config[11] = bar_dp * density;
    let bar = Config::decode(&sized.config)?;
    // FlorisImeSizing.smartbarUiHeight, from the settings rather than a search's bar.
    let bar_ui = match (bar.enabled, bar.extra() && bar.placement != 2) {
        (false, _) => 0.0,
        (true, true) => bar_dp * 2.0,
        (true, false) => bar_dp,
    };
    let search = flag(f::SEARCH);
    if search {
        gif_search(&mut sized.config, bar_dp * density);
    }
    let mut out = vec![0.0; p::SIZE];
    out[p::PANEL] = panel(a);
    out[p::ROW_PIXELS] = row_px as i32 as f32;
    out[p::SMARTBAR_PIXELS] = bar_px as i32 as f32;
    out[p::ROW_HEIGHT] = row_dp * density;
    out[p::KEYBOARD_PX] = sized.keyboard_dp * density;
    out[p::SMARTBAR_ROW] = bar_dp * density;
    place(a, &sized, sized.keyboard_dp + bar_ui, &mut out)?;
    out.extend_from_slice(&sized.config);
    Some(out)
}

/// What every placement reads: the density, the keyboard's dp height and the Smartbar as drawn.
struct Sized {
    density: f32,
    keyboard_dp: f32,
    config: Vec<f32>,
}

/// `FlorisImeSizing.keyboardUiHeight`'s rows: a characters keyboard's for the modes that share
/// its height, at least four.
fn rows(a: &[f32]) -> i32 {
    // KeyboardMode.value: characters, symbols, symbols 2 and numeric advanced.
    let character = matches!(a[f::MODE] as i32, 0 | 2 | 3 | 5);
    (a[if character {
        f::CHARACTER_ROWS
    } else {
        f::ROWS
    }] as i32)
        .max(4)
}

/// The row, box and column: every placement in `out`, from the panels' height `ime_dp`.
fn place(a: &[f32], s: &Sized, ime_dp: f32, out: &mut [f32]) -> Option<()> {
    let flag = |i: usize| a[i] != 0.0;
    let int = |i: usize| a[i] as i32;
    let d = s.density;
    let landscape = flag(f::LANDSCAPE);
    let (left, top) = (int(f::SAFE_LEFT), int(f::SAFE_TOP));
    let content = int(f::WIDTH) - left - int(f::SAFE_RIGHT);
    let (enabled, end) = (flag(f::ONE_HANDED), flag(f::ONE_HANDED_END));
    let w = one_handed(enabled, landscape, int(f::ONE_HANDED_SCALE), end);
    // A side's weight is its panel's: away from the keyboard, in portrait only.
    let (start_panel, end_panel) = (w[1] > 0.0, w[2] > 0.0);
    let mut weights = vec![];
    if start_panel {
        weights.push(w[1]);
    }
    weights.push(w[0]);
    if end_panel {
        weights.push(w[2]);
    }
    let widths = weighted(content, &weights);
    let keyboard_width = widths[usize::from(start_panel)];
    let bar = Config::decode(&s.config)?;
    let (strip, inset) = (bar.height(), bar.inset());
    let keyboard_height = round((s.keyboard_dp + inset / d) * d);
    let panel_height = round(ime_dp * d);
    let mut row = keyboard_height;
    let mut x = left;
    for (i, width) in widths.iter().enumerate() {
        let (at, height) = if i == usize::from(start_panel) {
            (p::KEYBOARD_LEFT, keyboard_height)
        } else {
            row = row.max(panel_height);
            (
                if i == 0 { p::START_LEFT } else { p::END_LEFT },
                panel_height,
            )
        };
        out[at..at + 4].copy_from_slice(&[x as f32, top as f32, *width as f32, height as f32]);
        x += width;
    }
    let offset = int(if landscape {
        f::OFFSET_LANDSCAPE
    } else {
        f::OFFSET_PORTRAIT
    });
    let box_height = top + row + round(offset as f32 * d) + int(f::SAFE_BOTTOM);
    column(a, box_height, out);
    surface(a, s, strip, keyboard_height, out);
    out[p::STRIP_PX] = strip / d * d;
    if bar.inline_visible() && out[p::PANEL] == 0.0 && !flag(f::SEARCH) {
        let layout = smartbar::layout(&bar, keyboard_width.max(1) as f32, &[], 0.0);
        let c = layout.center;
        let dp = |px: f32| round(px / d * d) as f32;
        out[p::INLINE_LEFT] = dp(c.left);
        out[p::INLINE_TOP] = dp(c.top - strip + inset);
        out[p::INLINE_WIDTH] = dp(c.right - c.left);
        out[p::INLINE_HEIGHT] = dp(c.bottom - c.top);
    }
    Some(())
}

/// `ImeUiWrapper`'s column: the view's height for its measure spec, and the box's top, under
/// the spacer that keeps the keyboard at the bottom.
fn column(a: &[f32], box_height: i32, out: &mut [f32]) {
    let spacer = !(a[f::FULLSCREEN] != 0.0 && a[f::EXTRACT_SHOWN] != 0.0);
    let spec = a[f::HEIGHT] as i32;
    let view = match a[f::HEIGHT_MODE] as i32 {
        2 => spec,
        1 if spacer => spec,
        1 => box_height.min(spec),
        _ => box_height,
    };
    out[p::VIEW_HEIGHT] = view as f32;
    out[p::BOX_TOP] = if spacer {
        (view - box_height).max(0)
    } else {
        0
    } as f32;
    out[p::BOX_HEIGHT] = box_height as f32;
    out[p::RISE] = if spacer { 0.0 } else { 1.0 };
}

/// `RustKeyboardLayout`'s surface: the keyboard and strip tall, or the screen less its bars
/// under the actions editor, bottom-aligned in the keyboard's box.
fn surface(a: &[f32], s: &Sized, strip: f32, keyboard_height: i32, out: &mut [f32]) {
    let d = s.density;
    let bars = a[f::NAVIGATION_BAR] as i32 + a[f::STATUS_BAR] as i32;
    let dp = if out[p::PANEL] == 3.0 {
        (a[f::DISPLAY_HEIGHT] as i32 - bars) as f32 / d
    } else {
        s.keyboard_dp + strip / d
    };
    let height = round(dp * d);
    out[p::SURFACE_TOP] = (keyboard_height - height) as f32;
    out[p::SURFACE_HEIGHT] = height as f32;
}

#[cfg(test)]
#[path = "ime_root_tests.rs"]
mod tests;
