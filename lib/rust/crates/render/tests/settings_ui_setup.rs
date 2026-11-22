//! SetupScreen drawn natively: the first screen a fresh activity opens, each step's state from the
//! input method and permission facts, opening steps by their headers, each step's button, the
//! permission request and its answers, coming back from the system's screens, finishing, the
//! TalkBack nodes, the focus order and the saved state.
use render::{
    settings_ui::{
        Action, App, Control, catalogue,
        setup::{
            self, NOTIFICATIONS, ROUTE, SET_UP, Step, Tap,
            paint::{ACTION_ID, HEADER_ID, PRIVACY_ID, REPOSITORY_ID, STATUS_ID, TEXT_ID},
        },
    },
    text::Text,
};
use wire::settings_ui::setup::{effect, fact};

const STRINGS: [(&str, &str); 14] = [
    ("setup__step__label", "Step {n} of {count}: {title}"),
    ("setup__step__done", "Done"),
    ("setup__step__current", "Current step"),
    ("setup__step__next", "Up next"),
    (
        "setup__step__later",
        "Not yet: finish the steps above first",
    ),
    ("setup__enable_ime__title", "Enable AudioWhisper"),
    ("setup__select_ime__title", "Select AudioWhisper"),
    (
        "setup__grant_notification_permission__title",
        "Permit Crash Report Notifications",
    ),
    ("setup__finish_up__title", "Finish Up"),
    (
        "setup__grant_notification_permission__btn",
        "Grant Permission",
    ),
    (
        "setup__notifications__allowed",
        "Notifications are allowed.",
    ),
    ("setup__notifications__off", "Notifications are off."),
    (
        "setup__notifications__settings_btn",
        "Open Notification Settings",
    ),
    (
        "florisboard__privacy_policy_url",
        "https://florisboard.org/legal/privacy/",
    ),
];

fn roboto() -> Text {
    Text::new([include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec()])
}
/// A fresh activity at 480 by 800 px with the keyboard `enabled` and `selected`, the stored
/// notification answer and the permission facts Kotlin reports, after START; drawn.
fn fresh(ime: (bool, bool), answer: &str, granted: &str, rationale: &str) -> (App, Text) {
    let mut a = App {
        config: [480.0, 800.0, 1.0, 1.0],
        ..App::default()
    };
    for (k, v) in STRINGS {
        a.strings.insert(k.into(), v.into());
    }
    a.prefs.insert(SET_UP.into(), "false".into());
    a.prefs.insert(NOTIFICATIONS.into(), answer.into());
    a.set_ime(ime.0, ime.1);
    report(&mut a, granted, rationale);
    a.start(None);
    let mut text = roboto();
    a.scene(&mut text);
    (a, text)
}
/// A FACTS report with the permission facts, as Kotlin sends after each resume.
fn report(a: &mut App, granted: &str, rationale: &str) {
    a.facts = [(fact::NOTIFICATIONS, granted), (fact::RATIONALE, rationale)]
        .into_iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
    a.setup_facts();
}
fn control(a: &App, id: i32) -> Option<&Control> {
    a.controls.iter().find(|c| c.id == id)
}
fn header(a: &App, step: Step) -> &Control {
    control(a, HEADER_ID + step.number() as i32).expect("every step has a header")
}
fn tap(a: &mut App, text: &mut Text, id: i32) -> bool {
    let done = a.activate(id);
    a.scene(text);
    done
}
/// CONFIG with the input method facts, and the frame its reply draws.
fn ime(a: &mut App, text: &mut Text, enabled: bool, selected: bool) {
    a.set_ime(enabled, selected);
    a.scene(text);
}
/// A PERMISSION packet, and the frame its reply draws.
fn answer(a: &mut App, text: &mut Text, granted: bool, rationale: bool) {
    a.permission_answer(&[granted.to_string(), rationale.to_string()])
        .unwrap();
    a.scene(text);
}
fn effects(a: &mut App) -> Vec<Vec<String>> {
    std::mem::take(&mut a.effects)
}
fn writes(effects: &[Vec<String>]) -> Vec<(String, String)> {
    effects
        .iter()
        .filter(|e| e[0] == "write")
        .map(|e| (e[1].clone(), e[2].clone()))
        .collect()
}

#[test]
fn a_first_run_opens_setup_and_a_finished_one_opens_home() {
    let (mut a, _) = fresh((false, false), "NOT_SET", "false", "false");
    assert_eq!(a.stack, [ROUTE]);
    assert_eq!(a.scrolls, [0.0]);
    // FlorisScreen's `navigationIconVisible = false`: no back arrow, and Back leaves the app.
    assert!(control(&a, 1).is_none());
    assert!(!a.has_back());
    a.back();
    assert_eq!(effects(&mut a).pop().unwrap(), ["finish"]);

    let mut done = App::default();
    done.prefs.insert(SET_UP.into(), "true".into());
    done.prefs.insert(NOTIFICATIONS.into(), "DENIED".into());
    done.start(None);
    assert_eq!(done.stack, [catalogue::HOME_ROUTE]);
    assert!(done.effects.is_empty());
}

#[test]
fn notifications_allowed_outside_setup_leave_the_stored_answer() {
    // Only the setup screen answered the question in Compose; home leaves a denial stored.
    let mut a = App::default();
    a.prefs.insert(SET_UP.into(), "true".into());
    a.prefs.insert(NOTIFICATIONS.into(), "DENIED".into());
    a.start(None);
    report(&mut a, "true", "false");
    assert_eq!(a.stack, [catalogue::HOME_ROUTE]);
    assert!(a.effects.is_empty());
    assert_eq!(a.value(NOTIFICATIONS), "DENIED");
}

#[test]
fn an_unanswered_notification_question_sends_a_finished_user_through_setup_again() {
    let mut a = App::default();
    a.prefs.insert(SET_UP.into(), "true".into());
    a.prefs.insert(NOTIFICATIONS.into(), "NOT_SET".into());
    a.start(None);
    assert_eq!(a.stack, [ROUTE]);
    assert_eq!(writes(&a.effects), [(SET_UP.into(), "false".into())]);
}

#[test]
fn a_settings_link_opens_its_screen_even_during_setup() {
    let mut a = App::default();
    a.prefs.insert(SET_UP.into(), "false".into());
    a.start(Some("settings/dictation"));
    assert_eq!(a.stack, [catalogue::HOME_ROUTE, "settings/dictation"]);
    // A link Rust does not draw falls back to Compose, as before.
    let mut b = App::default();
    b.start(Some("ext/edit/my.themes"));
    assert_eq!(b.effects.pop().unwrap()[0], "compose");
}

#[test]
fn each_step_opens_from_the_facts() {
    let cases = [
        ((false, false), "NOT_SET", "false", Step::Enable),
        ((false, true), "GRANTED", "true", Step::Enable),
        ((true, false), "NOT_SET", "false", Step::Select),
        ((true, true), "NOT_SET", "false", Step::Notifications),
        ((true, true), "DENIED", "false", Step::Finish),
        ((true, true), "GRANTED", "true", Step::Finish),
    ];
    for (ime, answer, granted, open) in cases {
        let (a, _) = fresh(ime, answer, granted, "true");
        assert_eq!(a.setup_step(), open, "{ime:?} {answer} {granted}");
        assert_eq!(a.auto_step(), open);
        assert_eq!(header(&a, open).role, "heading");
    }
}

#[test]
fn notifications_already_allowed_skip_the_question() {
    // Improvement: Compose asked even when the permission was already granted.
    let (mut a, _) = fresh((true, true), "NOT_SET", "true", "false");
    assert_eq!(a.setup_step(), Step::Finish);
    assert_eq!(a.value(NOTIFICATIONS), "GRANTED");
    assert_eq!(
        writes(&effects(&mut a)),
        [(NOTIFICATIONS.into(), "GRANTED".into())]
    );
}

#[test]
fn headers_read_their_number_state_and_whether_a_tap_opens_them() {
    let (mut a, mut text) = fresh((true, false), "NOT_SET", "false", "true");
    let first = header(&a, Step::Enable);
    assert_eq!(first.label, "Step 1 of 4: Enable AudioWhisper");
    assert_eq!(
        (first.summary.as_str(), first.role, first.enabled),
        ("Done", "button", true)
    );
    assert_eq!(first.action, Action::Setup(Tap::Header(Step::Enable)));
    let open = header(&a, Step::Select);
    assert_eq!(
        (open.summary.as_str(), open.role),
        ("Current step", "heading")
    );
    assert_eq!(open.action, Action::None);
    for later in [Step::Notifications, Step::Finish] {
        let h = header(&a, later);
        assert_eq!(
            (h.summary.as_str(), h.role, h.enabled),
            ("Not yet: finish the steps above first", "text", false)
        );
        assert!(
            !a.activate(HEADER_ID + later.number() as i32),
            "a later step stays shut"
        );
    }
    // A done step opens by its header; the first undone one is then next, and opens it again.
    assert!(tap(&mut a, &mut text, HEADER_ID + 1));
    assert_eq!(a.setup_step(), Step::Enable);
    assert_eq!(a.setup.manual, Some(Step::Enable));
    let next = header(&a, Step::Select);
    assert_eq!((next.summary.as_str(), next.role), ("Up next", "button"));
    assert!(tap(&mut a, &mut text, HEADER_ID + 2));
    assert_eq!(a.setup_step(), Step::Select);
    assert_eq!(a.setup.manual, None);
}

#[test]
fn each_steps_button_does_the_step() {
    let (mut a, mut text) = fresh((false, false), "NOT_SET", "false", "true");
    assert!(tap(&mut a, &mut text, ACTION_ID));
    assert_eq!(effects(&mut a), [["enable"]]);
    ime(&mut a, &mut text, true, false);
    effects(&mut a);
    assert!(tap(&mut a, &mut text, ACTION_ID));
    assert_eq!(effects(&mut a), [["picker"]]);
    ime(&mut a, &mut text, true, true);
    assert_eq!(a.setup_step(), Step::Notifications);
    assert_eq!(control(&a, ACTION_ID).unwrap().label, "Grant Permission");
    assert!(tap(&mut a, &mut text, ACTION_ID));
    assert_eq!(effects(&mut a), [[effect::ASK]]);
}

#[test]
fn finishing_writes_set_up_and_opens_home_with_no_way_back() {
    let (mut a, mut text) = fresh((true, true), "GRANTED", "true", "false");
    effects(&mut a);
    assert_eq!(a.setup_step(), Step::Finish);
    assert!(tap(&mut a, &mut text, ACTION_ID));
    assert_eq!(writes(&effects(&mut a)), [(SET_UP.into(), "true".into())]);
    assert_eq!(a.stack, [catalogue::HOME_ROUTE]);
    a.back();
    assert_eq!(effects(&mut a), [["finish"]]);
}

#[test]
fn the_keyboard_enabled_in_system_settings_brings_the_app_back() {
    let (mut a, mut text) = fresh((false, false), "NOT_SET", "false", "true");
    tap(&mut a, &mut text, ACTION_ID);
    effects(&mut a);
    a.set_ime(true, false);
    assert_eq!(effects(&mut a), [[effect::RETURN]]);
    assert_eq!(a.setup_step(), Step::Select);
    // Not again, and not for a keyboard enabled while the user was not sent there.
    a.set_ime(true, false);
    assert!(a.effects.is_empty());
    let (mut b, _) = fresh((false, false), "NOT_SET", "false", "true");
    b.set_ime(true, false);
    assert!(b.effects.is_empty());
}

#[test]
fn back_from_the_system_with_the_step_done_opens_the_next_even_when_reopened() {
    // Improvement: Compose kept a reopened step open after it was done again.
    let (mut a, mut text) = fresh((true, false), "NOT_SET", "false", "true");
    tap(&mut a, &mut text, HEADER_ID + 1);
    tap(&mut a, &mut text, ACTION_ID);
    assert_eq!(effects(&mut a), [["enable"]]);
    ime(&mut a, &mut text, true, false);
    assert_eq!(a.setup_step(), Step::Select);
    // Back from the picker with the keyboard still not selected: the step stays open.
    tap(&mut a, &mut text, ACTION_ID);
    ime(&mut a, &mut text, true, false);
    assert_eq!(a.setup_step(), Step::Select);
    assert_eq!(a.setup.away, Some((Step::Select, true)));
    ime(&mut a, &mut text, true, true);
    assert_eq!(a.setup_step(), Step::Notifications);
    assert_eq!(a.setup.away, None);
}

#[test]
fn the_permission_answer_decides_what_the_step_shows_next() {
    // Granted: stored as Compose stored it, and the last step opens.
    let (mut a, mut text) = fresh((true, true), "NOT_SET", "false", "false");
    tap(&mut a, &mut text, ACTION_ID);
    effects(&mut a);
    answer(&mut a, &mut text, true, false);
    assert_eq!(
        writes(&effects(&mut a)),
        [(NOTIFICATIONS.into(), "GRANTED".into())]
    );
    assert_eq!(a.setup_step(), Step::Finish);
    tap(&mut a, &mut text, HEADER_ID + 3);
    assert_eq!(
        control(&a, STATUS_ID).unwrap().label,
        "Notifications are allowed."
    );
    assert!(control(&a, ACTION_ID).is_none(), "nothing left to do");

    // Denied, Android would ask again: the step can ask again.
    let (mut b, mut text) = fresh((true, true), "NOT_SET", "false", "false");
    tap(&mut b, &mut text, ACTION_ID);
    answer(&mut b, &mut text, false, true);
    assert_eq!(b.value(NOTIFICATIONS), "DENIED");
    assert_eq!(b.setup_step(), Step::Finish);
    assert_eq!(
        b.setup.away, None,
        "answered, so the user is not away any more"
    );
    tap(&mut b, &mut text, HEADER_ID + 3);
    assert_eq!(b.notice(), setup::Notice::Denied);
    assert_eq!(
        control(&b, STATUS_ID).unwrap().label,
        "Notifications are off."
    );
    assert_eq!(control(&b, STATUS_ID).unwrap().role, "status");
    assert_eq!(control(&b, ACTION_ID).unwrap().label, "Grant Permission");
    effects(&mut b);
    tap(&mut b, &mut text, ACTION_ID);
    assert_eq!(effects(&mut b), [[effect::ASK]]);

    // Denied for good: improvement, the button opens the notification settings instead of a
    // request Android would drop.
    answer(&mut b, &mut text, false, false);
    tap(&mut b, &mut text, HEADER_ID + 3);
    assert_eq!(b.notice(), setup::Notice::Blocked);
    assert_eq!(
        control(&b, ACTION_ID).unwrap().label,
        "Open Notification Settings"
    );
    effects(&mut b);
    tap(&mut b, &mut text, ACTION_ID);
    assert_eq!(effects(&mut b), [[effect::NOTIFICATION_SETTINGS]]);
    // Allowed there: the next report answers the question.
    report(&mut b, "true", "false");
    assert_eq!(b.value(NOTIFICATIONS), "GRANTED");
    assert_eq!(b.setup.away, None);
}

#[test]
fn a_malformed_permission_answer_is_refused() {
    let (mut a, _) = fresh((true, true), "NOT_SET", "false", "false");
    assert!(a.permission_answer(&["true".into()]).is_err());
    assert!(
        a.permission_answer(&["yes".into(), "false".into()])
            .is_err()
    );
    assert!(a.permission_answer(&["true".into(), "".into()]).is_err());
    assert_eq!(a.value(NOTIFICATIONS), "NOT_SET");
}

#[test]
fn the_footer_opens_the_privacy_policy_and_the_repository() {
    let (mut a, mut text) = fresh((false, false), "NOT_SET", "false", "true");
    assert!(tap(&mut a, &mut text, PRIVACY_ID));
    assert_eq!(
        effects(&mut a),
        [["url", "https://florisboard.org/legal/privacy/"]]
    );
    assert!(tap(&mut a, &mut text, REPOSITORY_ID));
    assert_eq!(effects(&mut a), [["url", "florisboard__repo_url"]]);
}

#[test]
fn the_keyboard_moves_through_the_headers_a_tap_opens_then_the_button_and_footer() {
    let (a, _) = fresh((true, true), "NOT_SET", "false", "true");
    assert_eq!(
        a.setup_focus(),
        [
            HEADER_ID + 1,
            HEADER_ID + 2,
            ACTION_ID,
            PRIVACY_ID,
            REPOSITORY_ID
        ]
    );
    let (b, _) = fresh((true, true), "GRANTED", "true", "false");
    assert_eq!(
        b.setup_focus(),
        [
            HEADER_ID + 1,
            HEADER_ID + 2,
            HEADER_ID + 3,
            ACTION_ID,
            PRIVACY_ID,
            REPOSITORY_ID
        ]
    );
}

#[test]
fn the_open_step_fills_the_screen_and_a_taller_one_scrolls() {
    let (a, _) = fresh((false, false), "NOT_SET", "false", "true");
    assert_eq!(
        a.max_scroll, 0.0,
        "the steps fit as Compose's weight(1f) laid them out"
    );
    let footer = control(&a, PRIVACY_ID).unwrap().bounds;
    let viewport = control(&a, 4).unwrap().bounds;
    assert!(
        (footer.bottom - viewport.bottom).abs() <= 1.0,
        "{footer:?} {viewport:?}"
    );
    assert!(control(&a, TEXT_ID).is_some());
    let mut small = App {
        config: [480.0, 320.0, 1.0, 1.0],
        ..App::default()
    };
    small.start(None);
    let mut text = roboto();
    small.scene(&mut text);
    assert!(small.max_scroll > 0.0);
}

#[test]
fn the_chosen_step_survives_recreation() {
    let (mut a, mut text) = fresh((true, true), "DENIED", "false", "true");
    tap(&mut a, &mut text, HEADER_ID + 2);
    tap(&mut a, &mut text, ACTION_ID);
    assert_eq!(a.setup_saved(), "2|2|false");
    let state = App::parse_setup(&a.setup_saved()).unwrap();
    assert_eq!(state, a.setup);
    assert_eq!(
        App::parse_setup("0|0|false").unwrap(),
        setup::State::default()
    );
    for bad in [
        "",
        "1|2",
        "5|0|false",
        "1|0|maybe",
        "x|0|false",
        "1|2|false|3",
    ] {
        assert!(App::parse_setup(bad).is_err(), "{bad}");
    }
}

#[test]
fn opening_setup_again_starts_from_the_first_undone_step() {
    let (mut a, mut text) = fresh((true, true), "NOT_SET", "false", "true");
    tap(&mut a, &mut text, HEADER_ID + 1);
    a.navigate("settings/about");
    a.navigate(ROUTE);
    assert_eq!(a.stack, [ROUTE]);
    assert_eq!(a.setup_step(), Step::Notifications);
}
