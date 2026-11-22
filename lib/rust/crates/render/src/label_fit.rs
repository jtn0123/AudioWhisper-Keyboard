//! A label too big for its box, fitted inside it: a key's label and hint, a long-press choice,
//! and the press bubble's label. Compose drew each with `SnyggText`, which honours the theme's
//! `text-max-lines` and `text-overflow`: the text breaks into lines the box's width, keeps as many
//! as the theme allows, ends in an ellipsis where the theme asks for one, and is clipped to the
//! box. Where a theme sets neither rule, the label stays on one line and fits as a suggestion does
//! ([`crate::smartbar_fit`]): squeezed, then shrunk, then ellipsised.
//!
//! A label that fits draws exactly as it always did; the check costs a comparison of the layout it
//! already looks up. Only a label that does not fit is fitted, once for each label, font, box and
//! rule, and the fit is kept (`Raster::label_fits` counts how many were made).
use crate::{raster::Raster, smartbar_fit, style::Style, text::Text};
use keys::Rect;
use snygg::{
    resolve::{Properties, get},
    text,
    value::{Encoder, Value},
};
use std::{collections::HashMap, sync::Arc};

/// Snygg's `text-overflow`: how text cut short ends. `Unset` where the theme does not say.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Overflow {
    #[default]
    Unset,
    Clip,
    Ellipsis,
    Visible,
}

/// Snygg's `text-align` for the lines of a label that fills its box; `justify` draws as `start`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    Left,
    Right,
}

/// The rules a theme sets for text too big for its box. `max_lines` is `text-max-lines`, at
/// least one, with `none` as `i32::MAX`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextRules {
    pub max_lines: Option<i32>,
    pub overflow: Overflow,
    pub align: Align,
}

impl TextRules {
    pub fn from_properties(p: &Properties) -> Self {
        let word = |name: &str| match get(p, &text(name)) {
            Some(Value::Enum(Encoder::TextOverflow | Encoder::TextAlign, w)) => {
                String::from_utf16(w).ok()
            }
            _ => None,
        };
        Self {
            max_lines: match get(p, &text("text-max-lines")) {
                Some(Value::MaxLines(n)) => Some((*n).max(1)),
                _ => None,
            },
            overflow: match word("text-overflow").as_deref() {
                Some("clip") => Overflow::Clip,
                Some("ellipsis") => Overflow::Ellipsis,
                Some("visible") => Overflow::Visible,
                _ => Overflow::Unset,
            },
            align: match word("text-align").as_deref() {
                Some("center") => Align::Center,
                Some("end") => Align::End,
                Some("left") => Align::Left,
                Some("right") => Align::Right,
                _ => Align::Start,
            },
        }
    }

    /// Whether the theme says how this text breaks or ends; one it says neither for is squeezed.
    pub fn set(&self) -> bool {
        self.max_lines.is_some() || self.overflow != Overflow::Unset
    }
}

/// One line of a fitted label, placed from its paragraph's top left, its width unsqueezed.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub text: String,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

/// A label fitted to its box: its lines at `size`, `squeeze` as wide as they lay out (Android's
/// text scale X), in a paragraph `width` (unsqueezed) by `height`.
#[derive(Clone, Debug, PartialEq)]
pub struct Fitted {
    pub size: f32,
    pub squeeze: f32,
    pub width: f32,
    pub height: f32,
    pub lines: Vec<Line>,
    /// Compose's Text hangs a paragraph taller than its box from the box's top. A squeezed label
    /// stays centred.
    pub hangs: bool,
    /// `text-overflow: visible`: clipped to the key, not to the label's box.
    pub spills: bool,
}

/// `label` in `style` fitted to a box `room` wide and high that it does not fit whole.
pub fn fit(text: &mut Text, label: &str, style: &Style, room: [f32; 2]) -> Fitted {
    if style.rules.set() {
        wrapped(text, label, style, room)
    } else {
        squeezed(text, label, style, room)
    }
}

/// Where the theme says nothing: one line, squeezed to the width, then shrunk to the width and
/// the height, then ellipsised, as a suggestion fits its slot.
fn squeezed(text: &mut Text, label: &str, style: &Style, [width, height]: [f32; 2]) -> Fitted {
    let (mono, weight) = (style.mono, style.weight);
    let mut fit = smartbar_fit::fit_text(width, |scale| {
        text.measure(label, style.size * scale, mono, weight)
    });
    let tall = text.height(label, style.size * fit.scale, mono, weight);
    fit.scale = smartbar_fit::text_scale(fit.scale, tall, height);
    let size = style.size * fit.scale;
    let mut measure = |s: &str| text.measure(s, size, mono, weight) * fit.squeeze;
    let line = if measure(label) <= width {
        label.to_owned()
    } else {
        smartbar_fit::ellipsised(paragraph(label), width, measure)
    };
    let line = Line {
        width: text.measure(&line, size, mono, weight),
        height: text.height(&line, size, mono, weight),
        text: line,
        left: 0.0,
        top: 0.0,
    };
    Fitted {
        size,
        squeeze: fit.squeeze,
        width: line.width,
        height: line.height,
        lines: vec![line],
        hangs: false,
        spills: false,
    }
}

/// Compose's way: the lines the box's width breaks the label into, at most `text-max-lines` of
/// them and, with an ellipsis, at most as many as fit the height; where text is cut, the last
/// ends in an ellipsis. The paragraph is as wide as the label or the box, whichever is narrower,
/// and `text-align` places each line in it, `start` by the label's direction.
fn wrapped(text: &mut Text, label: &str, style: &Style, [width, height]: [f32; 2]) -> Fitted {
    let (size, mono, weight, rules) = (style.size, style.mono, style.weight, style.rules);
    let (ranges, rtl) = text.wrap(label, size, mono, weight, width);
    let tall = |text: &mut Text, s: &str| text.height(s, size, mono, weight);
    let max = usize::try_from(rules.max_lines.unwrap_or(i32::MAX)).unwrap_or(1);
    let ellipsis = rules.overflow == Overflow::Ellipsis;
    let mut keep = ranges.len().min(max);
    if ellipsis && keep > 1 {
        let mut total = 0.0;
        keep = ranges[..keep]
            .iter()
            .take_while(|r| {
                total += tall(text, &label[(*r).clone()]);
                total <= height
            })
            .count()
            .max(1);
    }
    let mut lines: Vec<String> = ranges[..keep]
        .iter()
        .map(|r| label[r.clone()].to_owned())
        .collect();
    if keep < ranges.len() && ellipsis {
        let rest = paragraph(&label[ranges[keep - 1].start..]);
        lines[keep - 1] =
            smartbar_fit::ellipsised(rest, width, |s| text.measure(s, size, mono, weight));
    }
    let paragraph = text.measure(label, size, mono, weight).min(width);
    let mut top = 0.0;
    let lines = lines
        .into_iter()
        .map(|line| {
            let (width, height) = (text.measure(&line, size, mono, weight), tall(text, &line));
            let left = aligned(rules.align, rtl, (paragraph - width).max(0.0));
            top += height;
            Line {
                text: line,
                left,
                top: top - height,
                width,
                height,
            }
        })
        .collect();
    Fitted {
        size,
        squeeze: 1.0,
        width: paragraph,
        height: top,
        lines,
        hangs: true,
        spills: rules.overflow == Overflow::Visible,
    }
}

/// A line's left in its paragraph, given the room it leaves there.
fn aligned(align: Align, rtl: bool, free: f32) -> f32 {
    let start = if rtl { free } else { 0.0 };
    match align {
        Align::Start => start,
        Align::End => free - start,
        Align::Center => free / 2.0,
        Align::Left => 0.0,
        Align::Right => free,
    }
}

/// `text` up to its first line break.
fn paragraph(text: &str) -> &str {
    text.split('\n').next().unwrap_or(text)
}

/// How a label that does not fit sits in its box: the share of the room it leaves on its left
/// and above it (Compose's alignment, 0.5 centred), and the key it may spill to when the theme
/// says `text-overflow: visible`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Place {
    pub bias: [f32; 2],
    pub outer: Rect,
}

impl Place {
    pub fn centred(outer: Rect) -> Self {
        Self {
            bias: [0.5; 2],
            outer,
        }
    }
}

type Key = (String, [u32; 3], bool, i32, TextRules);

/// The fits made, kept by label, font, box and rules, and how many were ever made.
#[derive(Default)]
pub(crate) struct Fits {
    kept: HashMap<Key, Arc<Fitted>>,
    made: usize,
}

impl Raster {
    /// How many labels were fitted to a box they did not fit, for want of a kept fit.
    pub fn label_fits(&self) -> usize {
        self.fits.made
    }

    /// `label` in `style` centred in `at`, as it always drew, where so it stays inside `room`, the
    /// box Compose gave its Text; else fitted to `room` and drawn inside it.
    pub(crate) fn contained_label(
        &mut self,
        label: &str,
        style: &Style,
        at: Rect,
        room: Rect,
        place: Place,
    ) -> Result<(), String> {
        let font = (style.size, style.mono, style.weight);
        let lines = style.rules.max_lines.map_or(usize::MAX, |n| n as usize);
        if self.label_within(label, font, 1.0, style.foreground, at, Some((room, lines)))? {
            return Ok(());
        }
        let fitted = self.fitted(label, style, room);
        let squeeze = fitted.squeeze;
        let free = [
            room.width() - fitted.width * squeeze,
            room.height() - fitted.height,
        ];
        let left = room.left + free[0].max(0.0) * place.bias[0];
        let top = room.top
            + match free[1] {
                free if free >= 0.0 => free * place.bias[1],
                _ if fitted.hangs => 0.0,
                free => free / 2.0,
            };
        let bounds = if fitted.spills { place.outer } else { room };
        self.context.push_clip_rect(&crate::raster::clip(bounds));
        let font = (fitted.size, style.mono, style.weight);
        for line in &fitted.lines {
            let r = Rect {
                left: left + line.left * squeeze,
                top: top + line.top,
                right: left + (line.left + line.width) * squeeze,
                bottom: top + line.top + line.height,
            };
            self.squeezed_label(&line.text, font, squeeze, style.foreground, r)?;
        }
        self.context.pop_clip();
        Ok(())
    }

    fn fitted(&mut self, label: &str, style: &Style, room: Rect) -> Arc<Fitted> {
        let bits = [style.size, room.width(), room.height()].map(f32::to_bits);
        let key = (
            label.to_owned(),
            bits,
            style.mono,
            style.weight,
            style.rules,
        );
        if let Some(fitted) = self.fits.kept.get(&key) {
            return fitted.clone();
        }
        // Bound the labels kept, as the layouts are, and as loosely: a theme too big for every
        // key overflows each label and hint, some 80 on a letter layout. No label is logged.
        if self.fits.kept.len() >= 256 {
            self.fits.kept.clear();
        }
        let fitted = Arc::new(fit(
            &mut self.text,
            label,
            style,
            [room.width(), room.height()],
        ));
        self.fits.made += 1;
        self.fits.kept.insert(key, fitted.clone());
        fitted
    }
}
