//! The languages screen and a language's words: routes, grouping, the alphabetical searchable
//! list with only Delete, delete and undo, the empty card, the actions and the outcomes.
use crate::common::*;
use render::settings_ui::{
    catalogue::words::{Act, Part},
    items::{EMPTY_ID, UNDO_ID, item_id},
    words::{
        self, FLORIS, FLORIS_ADD, FLORIS_EDIT, FLORIS_WORDS, Page, SYSTEM, SYSTEM_WORDS,
        form::GROUP_ID, group, place, route, screen_route,
    },
};
use wire::settings_ui::words::{dictionary, effect, outcome};

fn en_us() -> String {
    format!("{FLORIS_WORDS}/en_US")
}
fn titles(a: &render::settings_ui::App) -> Vec<String> {
    a.word_items().into_iter().map(|i| i.title).collect()
}

#[test]
fn routes_name_the_dictionary_and_page_both_ways() {
    let pages = [
        Page::Languages,
        Page::Words(None),
        Page::Words(Some("en_US".into())),
        Page::Add,
        Page::Edit(42),
    ];
    let screens = [FLORIS, FLORIS_WORDS, FLORIS_WORDS, FLORIS_ADD, FLORIS_EDIT];
    for (page, screen) in pages.iter().zip(screens) {
        let r = route(dictionary::FLORIS, page);
        assert_eq!(place(&r), Some((dictionary::FLORIS, page.clone())));
        assert_eq!(screen_route(&r), Some(screen));
        let s = route(dictionary::SYSTEM, page);
        assert_eq!(place(&s).unwrap().0, dictionary::SYSTEM);
    }
    assert_eq!(
        screen_route(&format!("{SYSTEM}/words/de_DE")),
        Some(SYSTEM_WORDS)
    );
    for bad in [
        "settings/dictionary/user-dictionary/OTHER",
        "settings/dictionary/user-dictionary/FLORIS/word/edit/x",
        "settings/dictionary/user-dictionary/FLORIS/elsewhere",
        "settings/dictionary",
    ] {
        assert_eq!(place(bad), None, "{bad}");
    }
}

#[test]
fn a_locale_is_grouped_as_floris_locale_reads_it() {
    let f = dictionary::FLORIS;
    for same in ["en_US", "en-us", "EN_us", "en-US"] {
        assert_eq!(group(f, Some(same)).as_deref(), Some("en_US"), "{same}");
    }
    assert_eq!(group(f, Some("iw")).as_deref(), Some("he"));
    assert_eq!(group(f, Some("in_ID")).as_deref(), Some("id_ID"));
    assert_eq!(group(f, Some("de")).as_deref(), Some("de"));
    // What the Floris dictionary's converter reads as no locale is all languages.
    for none in [None, Some("all"), Some("null"), Some("")] {
        assert_eq!(group(f, none), None, "{none:?}");
    }
    // The system's provider has only its null.
    assert_eq!(group(dictionary::SYSTEM, None), None);
    assert_eq!(
        group(dictionary::SYSTEM, Some("all")).as_deref(),
        Some("all")
    );
}

#[test]
fn the_languages_are_each_listed_once_all_languages_first_then_by_language() {
    let (mut a, mut text) = app(&[FLORIS], &sample());
    let groups = a.word_groups();
    let keys: Vec<_> = groups.iter().map(|g| g.key.as_deref()).collect();
    assert_eq!(keys, [None, Some("en_US"), Some("de_DE")]);
    let labels: Vec<_> = (0..3)
        .map(|k| control(&a, GROUP_ID + k).unwrap().label)
        .collect();
    assert_eq!(
        labels,
        [
            "For all languages",
            "English (United States)",
            "German (Germany)"
        ]
    );
    assert!(control(&a, GROUP_ID + 3).is_none());
    assert!(control(&a, EMPTY_ID).is_none());
    assert!(tap(&mut a, &mut text, GROUP_ID + 1));
    assert_eq!(a.stack.last().unwrap(), &en_us());
    // A word list is titled by its language, as Gboard's is.
    assert_eq!(a.word_title().as_deref(), Some("English (United States)"));
    a.back();
    assert_eq!(a.word_title(), None);
}

#[test]
fn an_empty_or_switched_off_dictionary_shows_the_card_and_disables_what_cannot_run() {
    let (a, _) = app(&[FLORIS], &[]);
    assert_eq!(
        control(&a, EMPTY_ID).unwrap().label,
        "settings__udm__no_words_in_dictionary"
    );
    let enabled = |a: &render::settings_ui::App, act: Act| {
        control(a, part_id(a, Part::Action(act))).unwrap().enabled
    };
    assert!(enabled(&a, Act::Add) && enabled(&a, Act::Import));
    assert!(!enabled(&a, Act::Export), "nothing to export");
    let (mut a, mut text) = app(&[FLORIS], &sample());
    assert!(enabled(&a, Act::Export));
    a.set_words(&report(dictionary::FLORIS, false, outcome::NONE, &[]))
        .unwrap();
    a.scene(&mut text);
    for act in [Act::Add, Act::Import, Act::Export] {
        assert!(!enabled(&a, act), "{act:?} runs with the dictionary off");
    }
}

#[test]
fn only_the_system_dictionary_offers_its_own_manager() {
    let (a, _) = app(&[FLORIS], &sample());
    assert!(!a.rows().iter().any(|r| matches!(
        r.kind,
        render::settings_ui::catalogue::Kind::Words(Part::Action(Act::Manager))
    )));
    let (mut a, mut text) = app(&[SYSTEM], &sample());
    let id = part_id(&a, Part::Action(Act::Manager));
    assert!(tap(&mut a, &mut text, id));
    assert_eq!(
        word_effects(&a),
        [vec![effect::MANAGER.to_string(), dictionary::SYSTEM.into()]]
    );
}

#[test]
fn a_languages_words_are_alphabetical_with_their_frequency_and_shortcut() {
    let (a, _) = app(&[FLORIS, &en_us()], &sample());
    assert_eq!(titles(&a), ["Quexor", "zorblat"]);
    let summaries: Vec<_> = a.word_items().into_iter().map(|i| i.summary).collect();
    assert_eq!(
        summaries,
        ["Frequency: 120 | Shortcut: qx", "Frequency: 200"]
    );
    let (a, _) = app(&[FLORIS, FLORIS_WORDS], &sample());
    assert_eq!(titles(&a), ["blivet", "snarf"]);
    assert_eq!(a.word_title().as_deref(), Some("For all languages"));
}

#[test]
fn a_word_has_only_delete_where_a_subtype_has_move_up_and_down_too() {
    let (mut a, _) = app(&[FLORIS, &en_us()], &sample());
    for title in ["Quexor", "zorblat"] {
        let slot = a.slot(
            &a.word_items()
                .iter()
                .find(|i| i.title == title)
                .unwrap()
                .key,
        );
        let delete = control(&a, item_id(slot, 3)).unwrap();
        assert_eq!((delete.role, delete.enabled), ("button", true));
        assert_eq!(delete.summary, title);
        assert!(
            control(&a, item_id(slot, 1)).is_none(),
            "{title} can move up"
        );
        assert!(
            control(&a, item_id(slot, 2)).is_none(),
            "{title} can move down"
        );
        assert_eq!(
            a.custom_actions_for(item_id(slot, 0)),
            [(item_id(slot, 3), "action__delete".to_string())]
        );
    }
    let search = part_id(&a, Part::Search);
    let (q, z) = (a.slot("2"), a.slot("1"));
    assert_eq!(
        focus_order(&mut a),
        [
            1,
            search,
            item_id(q, 0),
            item_id(q, 3),
            item_id(z, 0),
            item_id(z, 3),
            render::settings_ui::items::ADD_ID
        ]
    );
}

#[test]
fn the_search_narrows_by_word_or_shortcut_and_says_when_nothing_matches() {
    let (mut a, mut text) = app(&[FLORIS, &en_us()], &sample());
    let search = part_id(&a, Part::Search);
    assert!(a.line_field(search));
    for (term, found) in [
        ("QX", vec!["Quexor"]),
        (" zor ", vec!["zorblat"]),
        ("", vec!["Quexor", "zorblat"]),
    ] {
        a.words.search = term.into();
        a.scene(&mut text);
        assert_eq!(titles(&a), found, "{term}");
    }
    a.words.search = "nothing".into();
    a.scene(&mut text);
    assert!(titles(&a).is_empty());
    let id = part_id(&a, Part::NotFound);
    assert_eq!(
        control(&a, id).unwrap().label,
        "No word found for \u{201c}nothing\u{201d}"
    );
    assert!(control(&a, EMPTY_ID).is_none(), "the language has words");
    // The search starts empty each time the list opens.
    a.back();
    assert!(tap(&mut a, &mut text, GROUP_ID + 1));
    assert!(a.words.search.is_empty());
}

#[test]
fn delete_hides_the_word_at_once_asks_kotlin_and_undo_puts_it_back() {
    let (mut a, mut text) = app(&[FLORIS, &en_us()], &sample());
    let quexor = sample()[1].clone();
    let slot = a.slot("2");
    assert!(tap(&mut a, &mut text, item_id(slot, 3)));
    assert_eq!(
        word_effects(&a),
        [packet(
            effect::DELETE,
            dictionary::FLORIS,
            std::slice::from_ref(&quexor)
        )]
    );
    assert_eq!(titles(&a), ["zorblat"]);
    assert_eq!(a.focus, UNDO_ID);
    assert!(control(&a, UNDO_ID).is_some());
    // Kotlin's report confirms it; the undo stays.
    let rest: Vec<_> = sample().into_iter().filter(|w| w.id != 2).collect();
    a.set_words(&report(dictionary::FLORIS, true, outcome::NONE, &rest))
        .unwrap();
    a.effects.clear();
    assert!(tap(&mut a, &mut text, UNDO_ID));
    assert_eq!(
        word_effects(&a),
        [packet(effect::RESTORE, dictionary::FLORIS, &[quexor])]
    );
    assert_eq!(titles(&a), ["Quexor", "zorblat"]);
    assert!(a.list.deleted.is_none());
    assert!(control(&a, UNDO_ID).is_none());
}

#[test]
fn deleting_a_languages_last_word_stays_on_its_list_with_undo() {
    let route = format!("{FLORIS_WORDS}/de_DE");
    let (mut a, mut text) = app(&[FLORIS, &route], &sample());
    let slot = a.slot("4");
    assert!(tap(&mut a, &mut text, item_id(slot, 3)));
    assert_eq!(a.stack.last().unwrap(), &route);
    assert!(control(&a, EMPTY_ID).is_some());
    assert!(control(&a, UNDO_ID).is_some());
    // Still named, by the report that had a word in it.
    assert_eq!(a.word_title().as_deref(), Some("German (Germany)"));
    let mut rest = sample();
    rest.retain(|w| w.id != 4);
    a.set_words(&report(dictionary::FLORIS, true, outcome::NONE, &rest))
        .unwrap();
    assert_eq!(a.word_title().as_deref(), Some("German (Germany)"));
    assert!(tap(&mut a, &mut text, UNDO_ID));
    assert_eq!(titles(&a), ["frobnic"]);
    // A language no report has named is shown by its key.
    let (a, _) = app(&[FLORIS, &format!("{FLORIS_WORDS}/fr_FR")], &sample());
    assert_eq!(a.word_title().as_deref(), Some("fr_FR"));
}

#[test]
fn leaving_the_list_or_opening_a_word_forgets_the_undo() {
    let (mut a, mut text) = app(&[FLORIS, &en_us()], &sample());
    let slot = a.slot("2");
    assert!(tap(&mut a, &mut text, item_id(slot, 3)));
    a.back();
    assert!(a.list.deleted.is_none());
    // In all languages: delete one word, then open the other.
    assert!(tap(&mut a, &mut text, GROUP_ID));
    let slot = a.slot("3");
    assert!(tap(&mut a, &mut text, item_id(slot, 3)));
    assert!(a.list.deleted.is_some());
    let slot = a.slot("5");
    assert!(tap(&mut a, &mut text, item_id(slot, 0)));
    assert_eq!(a.stack.last().unwrap(), &format!("{FLORIS_EDIT}/5"));
    assert!(a.list.deleted.is_none());
}

#[test]
fn an_import_or_a_refused_change_is_told_and_a_bad_report_is_refused() {
    let (mut a, _) = app(&[FLORIS], &sample());
    a.strings.insert(
        "settings__udm__dictionary_import_success".into(),
        "Imported".into(),
    );
    a.set_words(&report(
        dictionary::FLORIS,
        true,
        outcome::IMPORTED,
        &sample(),
    ))
    .unwrap();
    a.set_words(&report(
        dictionary::FLORIS,
        true,
        outcome::FAILED,
        &sample(),
    ))
    .unwrap();
    assert_eq!(toasts(&a), ["Imported", "settings__udm__change_failed"]);
    let good = report(dictionary::FLORIS, true, outcome::NONE, &sample());
    let size = wire::settings_ui::words::row::SIZE;
    let mut bad_id = good.clone();
    bad_id[size + wire::settings_ui::words::row::ID] = "x".into();
    let mut word_first = good[size..2 * size].to_vec();
    word_first.extend(good[..size].iter().cloned());
    for bad in [vec![], good[..size - 1].to_vec(), bad_id, word_first] {
        assert!(a.set_words(&bad).is_err());
    }
    assert_eq!(a.word_report().unwrap().words, sample());
    assert_eq!(words::parse(&good).unwrap().locales.len(), 4);
}
