//! Material preview box: 36dp square minimum; long labels expand to intrinsic width.
pub fn size(density: f32, label_width: f32) -> [u16; 2] {
    let height = (36.0 * density).round().clamp(1.0, 8192.0) as u16;
    [
        label_width.ceil().max(height as f32).clamp(1.0, 8192.0) as u16,
        height,
    ]
}
