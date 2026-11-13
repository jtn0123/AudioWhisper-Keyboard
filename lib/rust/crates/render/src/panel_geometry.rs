//! LazyVerticalGrid's adaptive cells share whole pixels, distributing the remainder first.
pub fn cells(width: f32, minimum: f32) -> Vec<(f32, f32)> {
    let width = width.round().max(1.0) as usize;
    let count = (width / (minimum.round().max(1.0) as usize)).max(1);
    let base = width / count;
    let remainder = width % count;
    let mut x = 0;
    (0..count)
        .map(|i| {
            let size = base + usize::from(i < remainder);
            let cell = (x as f32, size as f32);
            x += size;
            cell
        })
        .collect()
}
pub fn history_moves(pinned: bool, pinned_strategy: i32, recent_strategy: i32) -> bool {
    (pinned && pinned_strategy >= 2) || recent_strategy >= 2
}
pub fn base(tones: &[i32], tone: i32) -> usize {
    tones.iter().position(|t| *t == tone).unwrap_or(0)
}
pub fn variants(tones: &[i32], tone: i32) -> Vec<usize> {
    if tones.len() <= 1 {
        vec![]
    } else {
        tones
            .iter()
            .enumerate()
            .filter(|(_, t)| **t != tone)
            .map(|(i, _)| i)
            .collect()
    }
}
