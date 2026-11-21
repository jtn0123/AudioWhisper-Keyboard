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
    for op in [
        ops::METADATA,
        ops::LAYOUT,
        ops::RESTORE,
        ops::FILE_NAME,
        ops::MANIFEST,
        ops::ITEMS,
        ops::SELECTION,
        ops::ARCHIVE,
    ] {
        assert!(call(op, &[]).is_err());
    }
    assert!(call(ops::RESTORE, &[text("bad"), text("0"), text("prefs")]).is_err());
    assert!(call(ops::ARCHIVE, &[text("root"), text("truncated")]).is_err());
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

/// Compose's clipboard TriStateCheckbox, counted over the clips a screen offers: Restore offers
/// only those a backup holds, so a backup with no images is all on with text and video.
#[test]
fn the_clipboard_checkbox_counts_only_offered_clips() {
    let [text_, images, video] = [3, 4, 5].map(|g| 1 << g);
    assert_eq!(clipboard_state(0b111, CLIPBOARD_MASK), ops::OFF);
    assert_eq!(clipboard_state(text_, CLIPBOARD_MASK), ops::MIXED);
    assert_eq!(clipboard_state(CLIPBOARD_MASK, CLIPBOARD_MASK), ops::ON);
    assert_eq!(clipboard_state(text_ | video, text_ | video), ops::ON);
    assert_eq!(clipboard_state(text_ | images, text_ | video), ops::MIXED);
    assert_eq!(clipboard_state(CLIPBOARD_MASK, 0b111), ops::OFF);
    assert_eq!(CLIPBOARD_MASK, text_ | images | video);
}
