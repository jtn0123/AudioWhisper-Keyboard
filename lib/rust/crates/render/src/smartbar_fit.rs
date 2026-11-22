//! How a suggestion fits its slot, Gboard's way: squeeze it, then shrink it, then ellipsise it.
//! One decision for both renderers: Compose calls [`fit_step`] through JNI with its own measure,
//! the Rust renderer through [`fit_text`] with Parley's. A key's label too big for its key fits
//! the same way where its theme says nothing else (`label_fit`), and both end in [`ellipsised`].
use unicode_segmentation::UnicodeSegmentation;

/// The narrowest a word is squeezed, as a fraction of its width at the same height. AOSP
/// LatinIME's suggestion strip squeezes to this and no further (`MIN_TEXT_XSCALE = 0.70f`,
/// `java/src/com/android/inputmethod/latin/suggestions/SuggestionStripLayoutHelper.java`, line 67,
/// platform/packages/inputmethods/LatinIME main at 127336e9f29d), where it ellipsises.
pub const MIN_TEXT_SQUEEZE: f32 = 0.70;
/// Past the squeeze floor a word shrinks, down to three quarters of its size, then ellipsises.
pub const MIN_TEXT_SCALE: f32 = 0.75;
/// Most measurements a renderer makes while fitting one word.
pub const TEXT_FIT_STEPS: usize = 3;

/// How a word draws: `squeeze` narrows its glyphs only (Android's text scale X, the height
/// kept), `scale` sizes its font.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextFit {
    pub squeeze: f32,
    pub scale: f32,
}
impl TextFit {
    pub const NATURAL: Self = Self {
        squeeze: 1.0,
        scale: 1.0,
    };
}

/// One fitting step. `measured` is the word's unsqueezed width at `fit.scale`. A squeeze is exact
/// arithmetic, so the word squeezes first, to the whole percent that fits, rounded down, never
/// below [`MIN_TEXT_SQUEEZE`]. Only a word that does not fit there shrinks ([`text_scale`] on its
/// squeezed width), and the renderer measures it again at the new scale. A word at both floors
/// that still does not fit ellipsises.
pub fn fit_step(fit: TextFit, measured: f32, available: f32) -> TextFit {
    if measured * fit.squeeze <= available || !measured.is_finite() || !available.is_finite() {
        return fit;
    }
    let squeeze = ((available.max(0.0) / measured * 100.0).floor() / 100.0)
        .max(MIN_TEXT_SQUEEZE)
        .min(fit.squeeze);
    if measured * squeeze <= available {
        return TextFit { squeeze, ..fit };
    }
    let scale = text_scale(fit.scale, measured * squeeze, available);
    TextFit { squeeze, scale }
}

/// One shrink step: the text measured `measured` wide at `scale` and has `available`. Width is
/// near linear in font size, so one step lands on the fit. Scales are whole percents, rounded
/// down, so a float error cannot leave a fitted word a hair too wide and ellipsised.
pub fn text_scale(scale: f32, measured: f32, available: f32) -> f32 {
    if measured <= available || !measured.is_finite() || !available.is_finite() {
        return scale;
    }
    let fit = (scale * available.max(0.0) / measured * 100.0).floor() / 100.0;
    fit.max(MIN_TEXT_SCALE).min(scale)
}

/// How a word draws in `available`, given the renderer's measure of its unsqueezed width at a
/// font scale. Measures once at full size, then again only when the font shrinks.
pub fn fit_text(available: f32, mut measure: impl FnMut(f32) -> f32) -> TextFit {
    let mut fit = TextFit::NATURAL;
    let mut measured = measure(fit.scale);
    let mut measures = 1;
    loop {
        let next = fit_step(fit, measured, available);
        if next == fit {
            return fit;
        }
        if next.scale != fit.scale {
            if measures == TEXT_FIT_STEPS {
                return next;
            }
            measured = measure(next.scale);
            measures += 1;
        }
        fit = next;
    }
}

/// The longest start of `text`, in whole graphemes, that fits `available` with an ellipsis after
/// it, and the ellipsis; empty where not even the ellipsis fits. A word too wide at both floors
/// ends so, and so does a key's label cut short. `measure` is the width of a string as drawn.
pub fn ellipsised(text: &str, available: f32, mut measure: impl FnMut(&str) -> f32) -> String {
    let mut ends = vec![0];
    ends.extend(text.grapheme_indices(true).map(|(i, s)| i + s.len()));
    let low = crate::smartbar::fit_prefix(ends.len(), available, |mid| {
        measure(&format!("{}…", &text[..ends[mid]]))
    });
    if low == 0 {
        String::new()
    } else {
        format!("{}…", &text[..ends[low - 1]])
    }
}
