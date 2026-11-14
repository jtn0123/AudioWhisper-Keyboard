use super::*;
#[test]
fn incompatible_versions_and_conflict_order_are_explicit() {
    assert_eq!(
        metadata(&text("vendor"), 63, 100),
        ["different_version", "invalid_metadata"]
    );
    assert_eq!(
        metadata(&text("vendor"), 101, 100),
        ["different_version", ""]
    );
    let paths = layout(&text("prefs"));
    let plan = restore(2, true, &paths, &vec![text("0"); 6]);
    assert_eq!(
        plan,
        vec![
            text("clear-directory"),
            paths[1].clone(),
            text("clear-clipboard"),
            vec![]
        ]
    );
}
#[test]
fn malformed_packets_fail_without_panics() {
    for op in 0..10 {
        if op != 6 && op != 8 {
            assert!(call(op, &[]).is_err());
        }
    }
    assert!(call(2, &[text("bad"), text("0"), text("prefs")]).is_err());
    assert!(call(9, &[text("root"), text("truncated")]).is_err());
}
