use render::{smartbar::inset, style::Style};
use snygg::{text, value::Value};
#[test]
fn borderless_theme_vertical_margins_reduce_background_without_changing_touch_bounds() {
    // key[code=10] and key[code=32] use "0dp 6dp" in the bundled Borderless themes.
    let style = Style::from_properties(
        &vec![(text("margin"), Value::Padding([0.0, 6.0, 0.0, 6.0]))],
        2.625,
        1.0,
    );
    assert_eq!(style.margin, [0.0, 16.0, 0.0, 16.0]);
    let visible = keys::Rect {
        left: 10.0,
        top: 0.0,
        right: 100.0,
        bottom: 124.0,
    };
    let background = inset(visible, style.margin);
    assert_eq!(background.height(), 92.0);
    assert_eq!(visible.height(), 124.0);
    assert_eq!((background.left, background.right), (10.0, 100.0));
}
