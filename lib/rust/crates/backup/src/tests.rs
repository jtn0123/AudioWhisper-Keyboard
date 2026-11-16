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

/// What the keyboard learned from typing stays on the phone, as Android's own backup rules keep
/// it (res/xml/data_extraction_rules.xml): only the settings (with "learn from typing" and "learn
/// how I tap" in them) and files under files/ime travel. learned-words.txt, kept-as-typed.txt,
/// tap-aim.txt and typing-counters.txt sit directly in files/, so no group can carry them.
#[test]
fn what_typing_taught_stays_on_the_phone() {
    let preferences = text("datastore/florisboard-app-prefs.preferences_pb");
    let paths = layout(&preferences);
    assert_eq!(paths[0], preferences);
    for path in paths.iter().map(|p| string(p)) {
        assert!(
            !path.starts_with("files/") || path.starts_with("files/ime/"),
            "{path}"
        );
        for learned in [
            "learned-words",
            "kept-as-typed",
            "tap-aim",
            "typing-counters",
        ] {
            assert!(!path.contains(learned), "{path}");
        }
    }
}
