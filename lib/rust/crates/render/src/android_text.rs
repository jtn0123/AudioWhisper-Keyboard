//! Android's layout of hinted UI text, as Compose draws it with `TextMotion.Static`.
//!
//! Minikin lays out non-linear text at the integer-truncated paint size: Skia's hinted advances
//! are whole pixels at that size, and GPOS kerning and letter spacing scale with it too. The
//! glyphs themselves still draw at the real size, from whole-pixel origins (no subpixel
//! positioning), and font ascent/descent stay linear in the real size. Measured glyph by glyph
//! with TextRunShaper on Android 16; see docs/stage4-followup3.md.
//!
//! Lines break greedily at ICU (UAX #14) opportunities, as Compose's default `LineBreak.Simple`
//! does, with trailing spaces hanging and over-long words split at cluster boundaries.
use icu_segmenter::{
    LineSegmenter,
    options::{LineBreakOptions, LineBreakStrictness},
};
use parley::FontData;
use skrifa::{
    MetadataProvider,
    instance::{LocationRef, Size},
    outline::{DrawSettings, Engine, HintingInstance, HintingOptions, SmoothMode, pen::NullPen},
    raw::types::{F2Dot14, GlyphId, Tag},
};
use std::{collections::HashMap, ops::Range};

/// The size Minikin lays non-linear text out at (`static_cast<int>(textSize)`).
pub fn layout_size(size: f32) -> f32 {
    size.trunc().max(1.0)
}

/// Hinted advance of one glyph: the base advance is whole pixels at the layout size (the
/// TrueType interpreter's when the font is hinted, else the rounded linear advance), while the
/// shaper's positioning adjustment (kerning) only scales to it.
pub fn advance(base: f32, hinted: Option<f32>, shaped: f32, size: f32) -> f32 {
    let k = layout_size(size) / size;
    hinted.unwrap_or_else(|| (base * k).round()) + (shaped - base) * k
}

type Face = (u64, u32, u32, Vec<i16>);

/// Advances from FreeType-equivalent TrueType hinting (`FT_LOAD_TARGET_NORMAL`), which Skia
/// reports for hinted fonts: phantom points can move a glyph's advance by a pixel.
#[derive(Default)]
pub struct Hinting {
    instances: HashMap<Face, Option<HintingInstance>>,
    advances: HashMap<(Face, u32), Option<f32>>,
}

impl std::fmt::Debug for Hinting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hinting").finish_non_exhaustive()
    }
}

impl Hinting {
    fn advance(&mut self, font: &FontData, ppem: f32, coords: &[i16], glyph: u32) -> Option<f32> {
        let face: Face = (font.data.id(), font.index, ppem.to_bits(), coords.to_vec());
        if let Some(advance) = self.advances.get(&(face.clone(), glyph)) {
            return *advance;
        }
        if self.advances.len() >= 16_384 {
            self.advances.clear();
            self.instances.clear();
        }
        let font_ref = skrifa::FontRef::from_index(font.data.as_ref(), font.index).ok();
        let outlines = font_ref.as_ref().map(|f| f.outline_glyphs());
        let instance = self.instances.entry(face.clone()).or_insert_with(|| {
            let font_ref = font_ref.as_ref()?;
            let hinted = [b"fpgm", b"prep"]
                .iter()
                .any(|t| font_ref.table_data(Tag::new(t)).is_some());
            let location: Vec<F2Dot14> = coords.iter().map(|c| F2Dot14::from_bits(*c)).collect();
            let options = HintingOptions {
                engine: Engine::Interpreter,
                target: SmoothMode::Normal.into(),
            };
            hinted
                .then(|| {
                    HintingInstance::new(
                        outlines.as_ref()?,
                        Size::new(ppem),
                        LocationRef::new(&location),
                        options,
                    )
                    .ok()
                })
                .flatten()
        });
        let advance = instance.as_ref().and_then(|instance| {
            outlines
                .as_ref()?
                .get(GlyphId::new(glyph))?
                .draw(DrawSettings::hinted(instance, false), &mut NullPen)
                .ok()?
                .advance_width
        });
        self.advances.insert((face, glyph), advance);
        advance
    }
}

/// One glyph run, positioned relative to the line's left edge and baseline.
#[derive(Clone, Debug)]
pub struct Run {
    pub font: FontData,
    pub coords: Vec<i16>,
    pub size: f32,
    /// Glyph id and pen position.
    pub glyphs: Vec<(u32, f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub range: Range<usize>,
    pub top: f32,
    pub baseline: f32,
    pub height: f32,
    /// Advance without hanging trailing spaces.
    pub width: f32,
    pub runs: Vec<Run>,
    /// Caret positions: text byte offset and x, including the line end.
    pub carets: Vec<(usize, f32)>,
}

#[derive(Clone, Debug, Default)]
pub struct Layout {
    pub lines: Vec<Line>,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug)]
struct Atom {
    range: Range<usize>,
    advance: f32,
    spacing: f32,
    space: bool,
    hard: bool,
    rtl: bool,
    paragraph: usize,
    run: usize,
    /// Glyph id, offsets, hinted advance and the kerning part of that advance, in visual
    /// order within the cluster.
    glyphs: Vec<(u32, f32, f32, f32, f32)>,
}

struct Paragraph {
    baseline: f32,
    height: f32,
}

/// Convert an unwrapped Parley layout, shaped at the real size without letter spacing, into
/// Android's wrapped layout. Runs are taken in logical order; mixed-direction lines keep
/// logical run order (settings strings are single-direction).
pub(crate) fn build(
    shaped: &parley::Layout<()>,
    text: &str,
    letter_spacing: f32,
    max_width: Option<f32>,
    hinting: &mut Hinting,
) -> Layout {
    let mut runs: Vec<Run> = Vec::new();
    let mut paragraphs = Vec::new();
    let mut atoms = Vec::new();
    for (paragraph, line) in shaped.lines().enumerate() {
        let m = line.metrics();
        paragraphs.push(Paragraph {
            baseline: m.baseline - m.block_min_coord,
            height: m.line_height,
        });
        let mut line_runs: Vec<_> = line.runs().collect();
        line_runs.sort_by_key(|r| r.text_range().start);
        for run in line_runs {
            runs.push(run_of(&run));
            let at = [paragraph, runs.len() - 1];
            shape_run(&run, letter_spacing, at, hinting, &mut atoms);
        }
    }
    let mut layout = Layout::default();
    let mut y = 0.0f64;
    for range in break_lines(text, &atoms, max_width) {
        let line_atoms = &atoms[range];
        let paragraph = line_atoms
            .first()
            .map_or_else(|| paragraphs.len().saturating_sub(1), |a| a.paragraph);
        let Some(p) = paragraphs.get(paragraph) else {
            continue;
        };
        let top = y.round() as f32;
        let start = line_atoms.first().map_or(text.len(), |a| a.range.start);
        let mut line = place(line_atoms, &runs, start);
        line.top = top;
        line.baseline = top + p.baseline;
        line.height = p.height;
        layout.width = layout.width.max(line.width);
        layout.lines.push(line);
        y += f64::from(p.height);
    }
    layout.height = y as f32;
    layout
}

fn run_of(run: &parley::Run<'_, ()>) -> Run {
    Run {
        font: run.font().clone(),
        coords: run
            .normalized_coords()
            .iter()
            .map(|c| c.to_bits())
            .collect(),
        size: run.font_size(),
        glyphs: Vec::new(),
    }
}

fn shape_run(
    run: &parley::Run<'_, ()>,
    letter_spacing: f32,
    [paragraph, index]: [usize; 2],
    hinting: &mut Hinting,
    atoms: &mut Vec<Atom>,
) {
    let coords: Vec<i16> = run
        .normalized_coords()
        .iter()
        .map(|c| c.to_bits())
        .collect();
    let size = run.font_size();
    let location: Vec<F2Dot14> = run
        .normalized_coords()
        .iter()
        .map(|c| F2Dot14::from_bits(c.to_bits()))
        .collect();
    let font = run.font();
    let metrics = skrifa::FontRef::from_index(font.data.as_ref(), font.index)
        .ok()
        .map(|f| f.glyph_metrics(Size::new(size), LocationRef::new(&location)));
    let k = layout_size(size) / size;
    for cluster in run.clusters() {
        // Parley splits a ligature into one cluster per character; Minikin spaces the
        // HarfBuzz cluster once, so continuations join the cluster that starts them.
        if cluster.is_ligature_continuation()
            && let Some(last) = atoms.last_mut().filter(|a| a.run == index)
        {
            last.range.end = last.range.end.max(cluster.text_range().end);
            continue;
        }
        let hard = cluster.is_hard_line_break();
        let glyphs: Vec<_> = cluster
            .glyphs()
            .map(|g| {
                let base = metrics
                    .as_ref()
                    .and_then(|m| m.advance_width(GlyphId::new(g.id)))
                    .unwrap_or(g.advance);
                let hinted = hinting.advance(font, layout_size(size), &coords, g.id);
                let advance = advance(base, hinted, g.advance, size);
                (g.id, g.x * k, g.y * k, advance, (g.advance - base) * k)
            })
            .collect();
        let spacing = if hard { 0.0 } else { letter_spacing * k };
        atoms.push(Atom {
            range: cluster.text_range(),
            advance: if hard {
                0.0
            } else {
                glyphs.iter().map(|g| g.3).sum::<f32>() + spacing
            },
            spacing,
            space: cluster.is_space_or_nbsp(),
            hard,
            rtl: run.is_rtl(),
            paragraph,
            run: index,
            glyphs,
        });
    }
}

/// Kerning from the cluster's last glyph toward whatever follows it.
fn tail_kern(atom: &Atom) -> f32 {
    atom.glyphs.last().map_or(0.0, |g| g.4)
}

/// Position one line's atoms; right-to-left runs are laid out reversed. Android shapes each
/// line on its own, so the line's last glyph keeps no kerning toward the next line, and
/// Minikin drops the half letter spacing outside the line's first and last clusters (its
/// LEFT_EDGE and RIGHT_EDGE run flags), in measurement and drawing alike.
fn place(atoms: &[Atom], runs: &[Run], start: usize) -> Line {
    let tail = atoms.iter().rposition(|a| !a.hard);
    let mut lefts = vec![0.0f32; atoms.len()];
    let mut widths = vec![0.0f32; atoms.len()];
    let mut line_runs: Vec<Run> = Vec::new();
    let mut pen = 0.0f32;
    let mut first = true;
    let mut i = 0;
    while i < atoms.len() {
        let run = atoms[i].run;
        let end = atoms[i..]
            .iter()
            .position(|a| a.run != run)
            .map_or(atoms.len(), |n| i + n);
        let mut order: Vec<usize> = (i..end).collect();
        if atoms[i].rtl {
            order.reverse();
        }
        let mut target = runs[run].clone();
        target.glyphs.clear();
        for n in order {
            let atom = &atoms[n];
            let lead = if first { 0.0 } else { atom.spacing / 2.0 };
            let kern = if Some(n) == tail {
                tail_kern(atom)
            } else {
                0.0
            };
            first = false;
            if !atom.hard {
                let mut x = pen + lead;
                for (id, dx, dy, advance, _) in &atom.glyphs {
                    target.glyphs.push((*id, x + dx, *dy));
                    x += advance;
                }
            }
            lefts[n] = pen;
            widths[n] = atom.advance - atom.spacing / 2.0 + lead - kern;
            pen += widths[n];
        }
        if !target.glyphs.is_empty() {
            line_runs.push(target);
        }
        i = end;
    }
    let caret = |n: usize, at_start: bool| {
        if atoms[n].rtl == at_start {
            lefts[n] + widths[n]
        } else {
            lefts[n]
        }
    };
    let mut carets: Vec<(usize, f32)> = (0..atoms.len())
        .map(|n| (atoms[n].range.start, caret(n, true)))
        .collect();
    let end = atoms.last().map_or(start, |a| a.range.end);
    carets.push((
        end,
        atoms.len().checked_sub(1).map_or(0.0, |n| caret(n, false)),
    ));
    let visible = atoms.iter().rposition(|a| !a.space && !a.hard);
    let hanging: f32 = visible.map_or(pen, |v| {
        widths[v + 1..].iter().sum::<f32>() + atoms[v].spacing / 2.0
    });
    Line {
        range: start..end,
        top: 0.0,
        baseline: 0.0,
        height: 0.0,
        width: pen - hanging,
        runs: line_runs,
        carets,
    }
}

/// Greedy line breaking over atoms; returns atom ranges per line.
fn break_lines(text: &str, atoms: &[Atom], max_width: Option<f32>) -> Vec<Range<usize>> {
    let mut options = LineBreakOptions::default();
    // ICU's default (untailored) rules, which LINE_BREAK_STYLE_NONE uses.
    options.strictness = Some(LineBreakStrictness::Strict);
    let opportunities: Vec<usize> = LineSegmenter::new_for_non_complex_scripts(options)
        .segment_str(text)
        .collect();
    // Minikin's WordBreaker drops ICU breaks after hard and soft hyphens: it leaves those to
    // automatic hyphenation, which Compose's default `Hyphens.None` turns off.
    let allowed = |at: usize| {
        opportunities.binary_search(&at).is_ok()
            && !text[..at].chars().next_back().is_some_and(breaking_hyphen)
    };
    let limit = max_width.unwrap_or(f32::INFINITY) + 1e-3;
    // Edge clusters lose their outer half spacing (see `place`).
    let edges =
        |start: usize, last: &Atom| (atoms[start].spacing + last.spacing) / 2.0 + tail_kern(last);
    let mut lines = Vec::new();
    let mut start = 0;
    let mut i = 0;
    let mut pen = 0.0f32;
    let mut candidate = None;
    while i < atoms.len() {
        let atom = &atoms[i];
        if atom.hard {
            lines.push(start..i + 1);
            start = i + 1;
            pen = 0.0;
            candidate = None;
            i += 1;
            continue;
        }
        if i > start && allowed(atom.range.start) {
            candidate = Some(i);
        }
        pen += atom.advance;
        if !atom.space && pen - edges(start, atom) > limit && i > start {
            let at = candidate.unwrap_or(i);
            lines.push(start..at);
            start = at;
            pen = 0.0;
            candidate = None;
            i = at;
            continue;
        }
        i += 1;
    }
    // A trailing hard break still owns an empty last line, as in StaticLayout.
    if start < atoms.len() || lines.is_empty() || atoms.last().is_some_and(|a| a.hard) {
        lines.push(start..atoms.len());
    }
    lines
}

/// Minikin's `Hyphenator::isLineBreakingHyphen`, plus the soft hyphen.
fn breaking_hyphen(c: char) -> bool {
    matches!(
        c,
        '\u{2d}'
            | '\u{ad}'
            | '\u{58a}'
            | '\u{5be}'
            | '\u{1400}'
            | '\u{2010}'
            | '\u{2013}'
            | '\u{2027}'
            | '\u{2e17}'
            | '\u{2e40}'
    )
}

impl Layout {
    /// The line holding a byte offset, and the caret x there.
    pub fn caret(&self, byte: usize) -> (usize, f32) {
        for (index, line) in self.lines.iter().enumerate() {
            let last = index + 1 == self.lines.len();
            if byte < line.range.end || last || line.range.is_empty() && byte == line.range.start {
                let x = line
                    .carets
                    .iter()
                    .rev()
                    .find(|(at, _)| *at <= byte)
                    .map_or(0.0, |c| c.1);
                return (index, x);
            }
        }
        (0, 0.0)
    }

    /// Nearest caret offset to a point.
    pub fn index_at(&self, x: f32, y: f32) -> usize {
        let Some(line) = self
            .lines
            .iter()
            .find(|l| y < l.top + l.height)
            .or(self.lines.last())
        else {
            return 0;
        };
        line.carets
            .iter()
            .min_by(|a, b| (a.1 - x).abs().total_cmp(&(b.1 - x).abs()))
            .map_or(line.range.start, |c| c.0)
    }

    /// Selection rectangles (left, top, right, bottom) between two byte offsets.
    pub fn selection(&self, a: usize, b: usize) -> Vec<[f32; 4]> {
        let (start, end) = (a.min(b), a.max(b));
        let mut rects = Vec::new();
        for line in &self.lines {
            let lo = start.max(line.range.start);
            let hi = end.min(line.range.end);
            if lo >= hi {
                continue;
            }
            let x = |at: usize| {
                line.carets
                    .iter()
                    .rev()
                    .find(|(c, _)| *c <= at)
                    .map_or(0.0, |c| c.1)
            };
            rects.push([x(lo), line.top, x(hi), line.top + line.height]);
        }
        rects
    }
}
