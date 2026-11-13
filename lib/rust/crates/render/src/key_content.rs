//! TextKeyButton's numeric phone keys use Compose BiasAlignment(-0.5, 0)/(0.5, 0).
use keys::Rect;

pub fn telpad(mode: &str, kind: &str) -> bool {
    mode == "phone" && kind == "numeric"
}

pub fn phone_label(area: Rect, metrics: [f32; 2], hint: bool) -> Rect {
    let [width, height] = metrics;
    let bias = if hint { 0.75 } else { 0.25 };
    let left = area.left + ((area.width() - width) * bias).round();
    let top = area.top + ((area.height() - height) * 0.5).round();
    Rect {
        left,
        top,
        right: left + width,
        bottom: top + height,
    }
}
