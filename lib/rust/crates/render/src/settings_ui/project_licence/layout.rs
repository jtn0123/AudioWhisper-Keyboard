//! The licence laid out as an unwrapped monospace Text: a line per newline, plus the empty last
//! line a trailing newline leaves, all one height. Each line is shaped with parley exactly as
//! the panel raster shapes the line it draws, so carets and highlights sit on the glyphs.
use super::super::App;
use crate::{
    android_text::{Layout, Line},
    text::Text,
};
use parley::PositionedLayoutItem;
use std::sync::Arc;

/// The text laid out at one pixel size.
#[derive(Debug)]
pub(in crate::settings_ui) struct Laid {
    pub size: f32,
    pub layout: Arc<Layout>,
}
/// Carets along one line drawn from `line`, its first byte at `start`: each cluster's start
/// and the line's end, in byte order.
fn carets(text: &mut Text, line: &str, start: usize, size: f32) -> Vec<(usize, f32)> {
    let shaped = text.layout_typography(line, size, true, 400, false, [0.0, 0.0]);
    let mut carets = vec![];
    let mut end = 0.0f32;
    for item in shaped.lines().flat_map(|l| l.items().collect::<Vec<_>>()) {
        if let PositionedLayoutItem::GlyphRun(run) = item {
            let mut x = run.offset();
            for cluster in run.run().visual_clusters() {
                carets.push((start + cluster.text_range().start, x));
                x += cluster.advance();
            }
            end = end.max(x);
        }
    }
    carets.sort_by_key(|c| c.0);
    carets.dedup_by_key(|c| c.0);
    carets.retain(|c| c.0 < start + line.len());
    carets.push((start + line.len(), end));
    carets
}
/// Lays `source` out at `size` pixels, unwrapped.
pub(super) fn lay_out(text: &mut Text, source: &str, size: f32) -> Layout {
    let reference = text.layout_typography("0", size, true, 400, false, [0.0, 0.0]);
    let height = reference.height();
    let baseline = reference
        .lines()
        .next()
        .map_or(height, |l| l.metrics().baseline);
    let mut layout = Layout::default();
    let mut start = 0;
    for (i, line) in source.split('\n').enumerate() {
        let end = (start + line.len() + 1).min(source.len());
        let carets = carets(text, line, start, size);
        let width = carets.last().map_or(0.0, |c| c.1);
        let top = i as f32 * height;
        layout.width = layout.width.max(width);
        layout.lines.push(Line {
            range: start..end.max(start),
            top,
            baseline: top + baseline,
            height,
            width,
            runs: vec![],
            carets,
        });
        start = end;
    }
    layout.height = layout.lines.len() as f32 * height;
    layout
}
impl App {
    /// The licence's layout at 10 sp, laid out again when the text or the size changes.
    pub(super) fn project_layout(&mut self, text: &mut Text) -> Option<Arc<Layout>> {
        let source = self.project_licence.text.clone()?;
        let size = self.sp(10);
        if self
            .project_licence
            .laid
            .as_ref()
            .is_none_or(|l| l.size != size)
        {
            let layout = Arc::new(lay_out(text, &source, size));
            self.project_licence.laid = Some(Laid { size, layout });
        }
        self.project_licence.laid.as_ref().map(|l| l.layout.clone())
    }
}
