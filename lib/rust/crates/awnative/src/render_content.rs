//! Font-independent oracle for the phone-key content placement used by the renderer.
use crate::jni::{Result, illegal};

/// Where a phone key's label goes in `bounds_and_metrics`: its four bounds and the label's two
/// metrics. Only the telephone pad's keys move it.
pub fn phone_label(
    mode: &[u16],
    kind: &[u16],
    bounds_and_metrics: &[f32],
    hint: bool,
) -> Result<Vec<f32>> {
    let mode = String::from_utf16_lossy(mode);
    let kind = String::from_utf16_lossy(kind);
    let v = bounds_and_metrics;
    if v.len() != 6 || v.iter().any(|x| !x.is_finite()) {
        return Err(illegal("invalid content bounds"));
    }
    let area = keys::Rect {
        left: v[0],
        top: v[1],
        right: v[2],
        bottom: v[3],
    };
    let r = if render::key_content::telpad(&mode, &kind) {
        render::key_content::phone_label(area, [v[4], v[5]], hint)
    } else {
        area
    };
    Ok(vec![r.left, r.top, r.right, r.bottom])
}
