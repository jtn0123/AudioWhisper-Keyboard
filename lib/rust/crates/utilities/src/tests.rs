use super::*;
#[test]
fn malformed_packets_and_collection_boundaries() {
    assert!(call(15, &[text("0"), text("8")]).is_err());
    assert!(call(10, &[text("0")]).is_err());
    assert!(call(6, &[text("-1"), vec![], vec![], vec![]]).is_err());
    assert!(call(11, &[vec![], vec![], vec![], vec![], vec![]]).is_err());
    for mode in 0..4 {
        assert_eq!(
            call(15, &[text(&mode.to_string()), text("0"), text("0")]).unwrap(),
            vec![text("0")]
        );
    }
}
#[test]
fn whitespace_and_utf16_are_not_lossily_normalized() {
    assert_eq!(
        reference::normalize(&[97, 115, 115, 101, 116, 115, 58, 0xd800]),
        vec![text("assets"), vec![47, 0xd800]]
    );
    assert_eq!(
        locale::parse(&text("en--variant-extra")),
        vec![text("en"), vec![], text("variant")]
    );
    assert_eq!(validation::validate(&text("word"), &[0xd800], 1, 255), "");
    assert_eq!(
        mime::compile(&text("a/x*")),
        vec![vec![], text("a"), text(r"x[^\s]+")]
    );
    assert!(mime::inputs(&[text("a/"), text("/b"), text("a/b/c")]).is_empty());
}
