//! Each change to the clipboard, as the Room-backed history made it.

use crate::clips::{Clip, Clips, Expiry, FileInfo, IMAGE, Limit, TEXT, VIDEO};

use super::golden::{check, clip, units};

const NOW: i64 = 1_700_000_000_000;
const NO_LIMIT: Limit = Limit {
    now: NOW,
    enabled: false,
    maximum: 0,
};

fn limit(maximum: i32) -> Limit {
    Limit {
        now: NOW,
        enabled: true,
        maximum,
    }
}

fn text(created: i64, text: &str) -> Clip {
    clip(0, TEXT, created, Some(text))
}

fn media(kind: i32, created: i64, uri: &str) -> Clip {
    Clip {
        uri: Some(units(uri)),
        ..clip(0, kind, created, None)
    }
}

fn ids(clips: &Clips) -> Vec<i64> {
    clips.history().iter().map(|clip| clip.id).collect()
}

#[test]
fn clips_get_growing_ids_and_show_newest_first() {
    let mut clips = Clips::default();
    for (created, body) in [(5, "a"), (9, "b"), (5, "c"), (7, "d")] {
        clips.add(text(created, body), NO_LIMIT);
    }
    assert_eq!(ids(&clips), [2, 4, 1, 3]); // equal times keep id order
    clips.delete(&[4], false, NO_LIMIT);
    assert_eq!(
        clips.add(text(1, "e"), NO_LIMIT),
        5,
        "an id is never given twice"
    );
}

#[test]
fn a_text_copied_again_moves_up_and_keeps_a_pin() {
    let mut clips = Clips::default();
    clips.add(text(1, "same"), NO_LIMIT);
    clips.add(text(2, "other"), NO_LIMIT);
    clips.set_pinned(1, true, NO_LIMIT);
    let id = clips.add(text(3, "same"), NO_LIMIT);
    assert_eq!(ids(&clips), [id, 2]);
    assert!(clips.history()[0].pinned);
    let pinned = Clip {
        pinned: true,
        ..text(4, "other")
    };
    clips.add(pinned, NO_LIMIT);
    assert!(
        clips.history()[0].pinned,
        "a pinned copy pins the clip it moves"
    );
}

#[test]
fn only_texts_move_and_no_text_matches_no_text() {
    let mut clips = Clips::default();
    clips.add(media(IMAGE, 1, "content://a/clips/images/1"), NO_LIMIT);
    clips.add(media(IMAGE, 2, "content://a/clips/images/1"), NO_LIMIT);
    assert_eq!(clips.items.len(), 2, "media never moves");
    clips.add(clip(0, TEXT, 3, None), NO_LIMIT);
    let image_without_text = media(IMAGE, 4, "content://a/clips/images/2");
    clips.add(image_without_text, NO_LIMIT);
    assert_eq!(
        clips.items.len(),
        3,
        "it replaced the text clip that had no text"
    );
    assert_eq!(clips.history()[0].kind, IMAGE);
}

#[test]
fn the_limit_removes_the_oldest_unpinned_clips() {
    let mut clips = Clips::default();
    clips.add(
        Clip {
            pinned: true,
            ..text(1, "pinned")
        },
        NO_LIMIT,
    );
    for created in 2..=6 {
        clips.add(text(created, &created.to_string()), limit(3));
    }
    assert_eq!(ids(&clips), [6, 5, 4, 1]);
    clips.enforce(limit(1));
    assert_eq!(ids(&clips), [6, 1]);
    clips.enforce(Limit {
        enabled: false,
        ..limit(0)
    });
    assert_eq!(ids(&clips), [6, 1]);
}

#[test]
fn restoring_adds_what_is_new_in_order() {
    let mut clips = Clips::default();
    clips.add(text(1, "kept"), NO_LIMIT);
    let backup = vec![
        Clip {
            id: 77,
            ..text(1, "kept")
        },
        text(2, "new"),
        text(2, "new"),
        Clip {
            pinned: true,
            ..text(1, "kept")
        },
    ];
    assert_eq!(clips.restore(backup, NO_LIMIT), 3);
    assert_eq!(
        clips.items.iter().map(|c| c.id).collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert!(clips.items[3].pinned);
}

#[test]
fn deleting_reports_media_nothing_refers_to() {
    let mut clips = Clips::default();
    let a = "content://a/clips/images/1";
    clips.add(media(IMAGE, 1, a), NO_LIMIT);
    clips.add(media(VIDEO, 2, "content://a/clips/videos/2"), NO_LIMIT);
    clips.add(
        Clip {
            pinned: true,
            ..media(IMAGE, 3, a)
        },
        NO_LIMIT,
    );
    clips.add(text(4, "t"), NO_LIMIT);
    check(
        &clips.delete(&[1], false, NO_LIMIT),
        &Vec::new(),
        "still used",
    );
    check(
        &clips.delete(&[3, 4], true, NO_LIMIT),
        &Vec::new(),
        "pinned kept",
    );
    check(
        &clips.delete(&[2, 3], false, NO_LIMIT),
        &vec![units("content://a/clips/videos/2"), units(a)],
        "orphans",
    );
    assert_eq!(clips.items.len(), 0);
}

#[test]
fn clearing_keeps_pins_unless_told() {
    let mut clips = Clips::default();
    clips.add(
        Clip {
            pinned: true,
            ..media(IMAGE, 1, "content://a/clips/images/1")
        },
        NO_LIMIT,
    );
    clips.add(media(IMAGE, 2, "content://a/clips/images/2"), NO_LIMIT);
    clips.add(text(3, "t"), NO_LIMIT);
    check(
        &clips.clear(true, NO_LIMIT),
        &vec![units("content://a/clips/images/2")],
        "unpinned media",
    );
    assert_eq!(ids(&clips), [1]);
    check(
        &clips.clear(false, NO_LIMIT),
        &vec![units("content://a/clips/images/1")],
        "all media",
    );
    assert!(clips.items.is_empty());
}

#[test]
fn old_and_sensitive_clips_expire() {
    let mut clips = Clips::default();
    clips.add(text(NOW - 3_600_001, "old"), NO_LIMIT);
    clips.add(
        Clip {
            pinned: true,
            ..text(NOW - 3_600_001, "old pinned")
        },
        NO_LIMIT,
    );
    clips.add(
        Clip {
            sensitive: true,
            ..text(NOW - 61_000, "secret")
        },
        NO_LIMIT,
    );
    clips.add(text(NOW, "fresh"), NO_LIMIT);
    let expiry = Expiry {
        old: true,
        minutes: 60,
        sensitive: false,
        seconds: 60,
    };
    clips.expire(expiry, NO_LIMIT);
    assert_eq!(ids(&clips), [4, 3, 2]);
    clips.expire(
        Expiry {
            old: false,
            sensitive: true,
            ..expiry
        },
        NO_LIMIT,
    );
    assert_eq!(ids(&clips), [4, 2]);
}

#[test]
fn pinning_changes_one_clip_and_an_unknown_one_none() {
    let mut clips = Clips::default();
    clips.add(text(1, "a"), NO_LIMIT);
    let before = clips.clone();
    clips.set_pinned(9, true, NO_LIMIT);
    check(&clips, &before, "unknown id");
    clips.set_pinned(1, true, NO_LIMIT);
    clips.set_pinned(1, false, NO_LIMIT);
    check(&clips, &before, "pinned and unpinned");
}

#[test]
fn file_records_are_kept_by_id() {
    let mut clips = Clips::default();
    let file = |id, size| FileInfo {
        id,
        size,
        ..FileInfo::default()
    };
    clips.add_file(file(1, 10));
    clips.add_file(file(2, 20));
    clips.add_file(file(1, 11));
    assert_eq!(clips.file(1).map(|f| f.size), Some(11));
    clips.remove_file(1);
    assert!(clips.file(1).is_none());
    assert_eq!(clips.files.len(), 1);
}
