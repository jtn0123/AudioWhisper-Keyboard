//! The page a link to a route Rust does not draw opens (compose-removal step 12): where Compose's
//! fallback activity once opened, Rust says the page does not exist and offers the way home.
use render::{
    settings_ui::{
        Action, App, Control,
        catalogue::{self, HOME_ROUTE, NOT_FOUND_ROUTE},
        setup::{NOTIFICATIONS, SET_UP},
    },
    text::Text,
};

const TITLE: &str = "settings__not_found__title";
const MESSAGE: &str = "settings__not_found__message";
const HOME: &str = "settings__not_found__home";

fn roboto() -> Text {
    Text::new([include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec()])
}
/// An activity at 480 by 800 px on `routes` in order, with the page's English strings; drawn.
fn opened(routes: &[&str]) -> (App, Text) {
    let mut a = App {
        config: [480.0, 800.0, 1.0, 1.0],
        ..App::default()
    };
    for (key, value) in [
        (TITLE, "Page not found"),
        (
            MESSAGE,
            "This link goes to a settings page that this version of AudioWhisper Keyboard doesn't have.",
        ),
        (HOME, "Go to the settings home"),
    ] {
        a.strings.insert(key.into(), value.into());
    }
    for route in routes {
        a.navigate(route);
    }
    let mut text = roboto();
    a.scene(&mut text);
    (a, text)
}
fn by_role<'a>(a: &'a App, role: &str) -> Vec<&'a Control> {
    a.controls.iter().filter(|c| c.role == role).collect()
}

#[test]
fn an_unknown_link_opens_the_page_over_what_was_open_and_back_returns_there() {
    let (mut a, _) = opened(&["settings/keyboard", "settings/no-such-page"]);
    assert_eq!(a.stack, [HOME_ROUTE, "settings/keyboard", NOT_FOUND_ROUTE]);
    // Nothing leaves Rust: no Compose activity, no effect at all.
    assert!(a.effects.is_empty(), "{:?}", a.effects);
    a.back();
    assert_eq!(a.stack, [HOME_ROUTE, "settings/keyboard"]);
}

#[test]
fn the_page_says_the_link_has_no_page_and_offers_the_settings_home() {
    let (mut a, mut text) = opened(&["ext/no-such-page"]);
    let scroll = by_role(&a, "scroll");
    assert_eq!(scroll[0].label, "Page not found");
    let texts = by_role(&a, "text");
    assert_eq!(texts.len(), 1);
    assert!(
        texts[0]
            .label
            .starts_with("This link goes to a settings page")
    );
    let buttons: Vec<_> = by_role(&a, "button")
        .into_iter()
        .map(|c| (c.label.as_str(), c.action.clone()))
        .collect();
    assert_eq!(
        buttons,
        [
            ("Go to the settings home", Action::Route(HOME_ROUTE.into())),
            ("settings__navigate_back", Action::Back),
        ]
    );
    // The button opens home with nothing under it, as a home link does.
    let home = by_role(&a, "button")[0].id;
    assert!(a.activate(home));
    a.scene(&mut text);
    assert_eq!(a.stack, [HOME_ROUTE]);
}

/// The route is text another app chose; the page never shows it.
#[test]
fn the_page_never_repeats_the_link() {
    let (a, _) = opened(&["settings/Call 555-0100 to fix your phone"]);
    assert_eq!(a.stack.last().map(String::as_str), Some(NOT_FOUND_ROUTE));
    for c in &a.controls {
        assert!(
            !c.label.contains("555") && !c.summary.contains("555"),
            "{c:?}"
        );
    }
}

#[test]
fn a_second_unknown_link_keeps_one_page() {
    let (a, _) = opened(&["settings/a", "settings/b"]);
    assert_eq!(a.stack, [HOME_ROUTE, NOT_FOUND_ROUTE]);
}

/// A fresh activity for an unknown link: the page over home, set up or not, after START's own rule
/// for an unanswered notification question.
#[test]
fn start_with_an_unknown_link_opens_the_page_over_home() {
    for (set_up, answer, writes) in [
        ("true", "GRANTED", 0),
        ("false", "GRANTED", 0),
        ("true", "NOT_SET", 1),
    ] {
        let mut a = App::default();
        a.prefs.insert(SET_UP.into(), set_up.into());
        a.prefs.insert(NOTIFICATIONS.into(), answer.into());
        a.start(Some("settings/gone"));
        assert_eq!(a.stack, [HOME_ROUTE, NOT_FOUND_ROUTE], "{set_up} {answer}");
        assert_eq!(
            a.effects.iter().filter(|e| e[0] == "write").count(),
            writes,
            "{set_up} {answer}"
        );
        assert!(a.effects.iter().all(|e| e[0] == "write"), "{:?}", a.effects);
    }
}

#[test]
fn enable_stays_an_effect_and_not_a_page() {
    let (mut a, _) = opened(&["settings/keyboard"]);
    a.navigate("@enable");
    assert_eq!(a.effects, [["enable"]]);
    assert_eq!(a.stack, [HOME_ROUTE, "settings/keyboard"]);
}

#[test]
fn the_page_is_a_native_screen_whose_strings_kotlin_resolves() {
    assert!(catalogue::native(NOT_FOUND_ROUTE));
    assert!(catalogue::native_routes().contains(&NOT_FOUND_ROUTE));
    let screen = catalogue::screen(NOT_FOUND_ROUTE).unwrap();
    assert_eq!(screen.title, TITLE);
    let keys = App::string_keys();
    for key in [TITLE, MESSAGE, HOME] {
        assert!(keys.contains(&key), "{key}");
    }
}
