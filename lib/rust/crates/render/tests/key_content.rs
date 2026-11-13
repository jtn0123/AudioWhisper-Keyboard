use keys::Rect;
use render::key_content::{phone_label, telpad};

#[test]
fn only_phone_numeric_keys_put_digits_and_hints_side_by_side() {
    for mode in [
        "characters",
        "symbols",
        "symbols2",
        "numeric",
        "numeric_advanced",
        "phone",
        "phone2",
    ] {
        assert_eq!(telpad(mode, "numeric"), mode == "phone");
        assert!(!telpad(mode, "character"));
    }
    let area = Rect {
        left: 0.,
        top: 0.,
        right: 260.,
        bottom: 150.,
    };
    let digit = phone_label(area, [32., 68.], false);
    let hint = phone_label(area, [57., 37.], true);
    assert_eq!((digit.left, digit.top), (57., 41.));
    assert_eq!((hint.left, hint.top), (152., 57.));
    assert!(digit.right < hint.left);
}
