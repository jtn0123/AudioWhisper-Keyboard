use super::*;

fn u(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

const ON: Typing = Typing {
    auto_space_punctuation: true,
    raw_editor: false,
    key_variation: NORMAL_VARIATION,
    selection_valid: true,
    selection_start: 5,
};

struct Texts {
    preceding_auto: Vec<u16>,
    following_auto: Vec<u16>,
    preceding_phantom: Vec<u16>,
    following_phantom: Vec<u16>,
    language: Vec<u16>,
    before_1: Vec<u16>,
    before_3: Vec<u16>,
    word: Vec<u16>,
}

impl Texts {
    fn new(before: &str, word: &str) -> Self {
        Texts {
            preceding_auto: u(".,?!"),
            following_auto: u("¿"),
            preceding_phantom: u(".,?!"),
            following_phantom: u("("),
            language: u("en"),
            before_1: u(&before[before.len() - 1..]),
            before_3: u(before),
            word: u(word),
        }
    }

    fn editor(&self) -> Editor<'_> {
        Editor {
            rule: Rule {
                preceding_auto_space: &self.preceding_auto,
                following_auto_space: &self.following_auto,
                preceding_phantom_space: &self.preceding_phantom,
                following_phantom_space: &self.following_phantom,
            },
            language: &self.language,
            before_1: Some(&self.before_1),
            before_3: Some(&self.before_3),
            current_word: Some(&self.word),
        }
    }
}

#[test]
fn a_full_stop_after_a_word_is_auto_spaced() {
    let texts = Texts::new("cat", "cat");
    let commit = commit_char(&ON, &u("."), 0, 0, Some(&texts.editor())).unwrap();
    assert_eq!(
        commit,
        Commit {
            delete_previous_space: false,
            space_before: false,
            space_after: true,
            auto_space: state::auto_space_active(true),
            phantom_space: state::INACTIVE,
        }
    );
}

#[test]
fn a_comma_after_an_auto_space_takes_it_back() {
    let texts = Texts::new("t. ", "");
    let auto = state::auto_space_active(true);
    let commit = commit_char(&ON, &u(","), auto, 0, Some(&texts.editor())).unwrap();
    assert!(commit.delete_previous_space && commit.space_after);
    // Without the flag the space counts, and nothing is spaced.
    let commit = commit_char(&ON, &u(","), 0, 0, Some(&texts.editor())).unwrap();
    assert!(!commit.delete_previous_space && !commit.space_after);
    assert_eq!(commit.auto_space, state::INACTIVE);
}

#[test]
fn numbers_are_not_auto_spaced() {
    let texts = Texts::new("3", "3");
    let commit = commit_char(&ON, &u("."), 0, 0, Some(&texts.editor())).unwrap();
    assert!(!commit.space_after);
}

#[test]
fn an_opening_mark_is_auto_spaced_before() {
    let texts = Texts::new("a", "a");
    assert_eq!(
        auto_space_before(&ON, &u("¿"), Some(&texts.editor())),
        Ok(true)
    );
    let texts = Texts::new("a ", "");
    assert_eq!(
        auto_space_before(&ON, &u("¿"), Some(&texts.editor())),
        Ok(false)
    );
}

#[test]
fn auto_spacing_is_off_by_setting_editor_or_variation() {
    let texts = Texts::new("cat", "cat");
    for typing in [
        Typing {
            auto_space_punctuation: false,
            ..ON
        },
        Typing {
            raw_editor: true,
            ..ON
        },
        Typing {
            key_variation: 1,
            ..ON
        },
    ] {
        assert_eq!(auto_space_after(&typing, &u("."), 0, None), Ok(false));
        assert_eq!(
            auto_space_before(&typing, &u("¿"), Some(&texts.editor())),
            Ok(false)
        );
    }
}

#[test]
fn a_rule_asks_for_the_text_only_once_it_needs_it() {
    assert_eq!(
        commit_char(&ON, &u("a"), 0, 0, None),
        Err(NEEDS_BEFORE_1 | NEEDS_BEFORE_3)
    );
    let off = Typing {
        auto_space_punctuation: false,
        ..ON
    };
    assert!(commit_char(&off, &u("a"), 0, 0, None).is_ok());
    assert_eq!(commit_char(&off, &u("a"), 1, 1, None), Err(NEEDS_BEFORE_1));
    assert_eq!(
        commit_char(&ON, &u(""), 1, 1, None).map(|c| c.space_before),
        Ok(false)
    );
    assert_eq!(
        phantom_space_before(&ON, &u("a"), 0, true, None),
        Err(NEEDS_BEFORE_1)
    );
    let editor = Editor {
        before_1: Some(&[0x61]),
        ..Editor::default()
    };
    assert_eq!(
        auto_space_after(&ON, &u("."), 0, Some(&editor)),
        Err(NEEDS_BEFORE_3)
    );
}

#[test]
fn a_phantom_space_goes_between_words() {
    let texts = Texts::new("cat", "");
    let editor = texts.editor();
    let owed = state::phantom_space_active(false, true);
    assert_eq!(
        phantom_space_before(&ON, &u("d"), owed, false, Some(&editor)),
        Ok(true)
    );
    assert_eq!(
        phantom_space_before(&ON, &u("("), owed, false, Some(&editor)),
        Ok(true)
    );
    assert_eq!(
        phantom_space_before(&ON, &u("."), owed, false, Some(&editor)),
        Ok(false)
    );
    assert_eq!(
        phantom_space_before(&ON, &u("d"), 0, false, Some(&editor)),
        Ok(false)
    );
    assert_eq!(
        phantom_space_before(&ON, &u("d"), 0, true, Some(&editor)),
        Ok(true)
    );
    let start = Typing {
        selection_start: 0,
        ..ON
    };
    assert_eq!(
        phantom_space_before(&start, &u("d"), owed, false, None),
        Ok(false)
    );
    let invalid = Typing {
        selection_valid: false,
        ..ON
    };
    assert_eq!(
        phantom_space_before(&invalid, &u("d"), owed, false, None),
        Ok(false)
    );
    let commit = commit_char(&ON, &u("d"), 0, owed, Some(&editor)).unwrap();
    assert!(commit.space_before && !commit.space_after);
    assert_eq!(commit.phantom_space, state::INACTIVE);
}

#[test]
fn no_phantom_space_is_typed_in_an_address_url_or_password() {
    let texts = Texts::new("cat", "");
    let owed = state::phantom_space_active(false, true);
    // KeyVariation: e-mail address, password, URL; then "all", which no field gets.
    for key_variation in [1, 3, 4, 0] {
        let typing = Typing {
            key_variation,
            ..ON
        };
        for force_active in [false, true] {
            assert_eq!(
                phantom_space_before(&typing, &u("d"), owed, force_active, None),
                Ok(false),
                "{key_variation} {force_active}"
            );
        }
        let commit = commit_char(&typing, &u("d"), 0, owed, Some(&texts.editor())).unwrap();
        assert!(!commit.space_before, "{key_variation}");
    }
}

#[test]
fn a_phantom_space_follows_listed_symbols_not_spaces() {
    let texts = Texts::new("a.", "");
    let owed = state::phantom_space_active(false, true);
    assert_eq!(
        phantom_space_before(&ON, &u("d"), owed, false, Some(&texts.editor())),
        Ok(true)
    );
    let texts = Texts::new("a ", "");
    assert_eq!(
        phantom_space_before(&ON, &u("d"), owed, false, Some(&texts.editor())),
        Ok(false)
    );
}

#[test]
fn unspaced_languages_get_no_phantom_space_but_japanese_does() {
    for (language, spaced) in [
        ("zh", false),
        ("ko", false),
        ("jp", false),
        ("th", false),
        ("ja", true),
        ("en", true),
        ("z", true),
        ("zhh", true),
        ("", true),
    ] {
        assert_eq!(supports_auto_space(&u(language)), spaced, "{language}");
        let mut texts = Texts::new("cat", "");
        texts.language = u(language);
        assert_eq!(
            phantom_space_before(&ON, &u("d"), 1, false, Some(&texts.editor())),
            Ok(spaced)
        );
    }
}
