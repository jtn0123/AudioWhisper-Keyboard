//! The theme's choice, session, rows and chip colours.

use crate::calls::{Error, call, rule};
use crate::choose::{Facts, Mode, Name};
use crate::inline::{BACKGROUND, FOREGROUND, Palette, chip};
use crate::session::{Component, Extension, Index, Session, Step};
use snygg::text;
use wire::ime_theme::{accent, inline, kind, step, update};

fn name(extension: &str, component: &str) -> Name {
    Name {
        extension: text(extension),
        component: text(component),
    }
}

fn facts(mode: Mode) -> Facts {
    Facts {
        mode,
        day: name("ext", "day"),
        night: name("ext", "night"),
        sunrise: 100,
        sunset: 200,
        now: 150,
        system_night: false,
        preview: None,
    }
}

fn index() -> Index {
    let component = |id: &str, stylesheet: &str| Component {
        id: text(id),
        stylesheet: text(stylesheet),
    };
    Index {
        keyboards: vec![text("keyboard")],
        themes: vec![
            Extension {
                id: text("ext"),
                source: true,
                components: vec![component("day", "  "), component("night", "n.json")],
            },
            Extension {
                id: text("keyboard"),
                source: true,
                components: vec![component("day", "")],
            },
            Extension {
                id: text("bare"),
                source: false,
                components: vec![component("day", "")],
            },
        ],
    }
}

#[test]
fn the_mode_chooses_day_or_night_and_a_preview_wins() {
    let mut f = facts(Mode::AlwaysDay);
    assert_eq!(f.active(), &f.day);
    f.mode = Mode::AlwaysNight;
    assert_eq!(f.active(), &f.night);
    f.mode = Mode::FollowSystem;
    assert_eq!(f.active(), &f.day);
    f.system_night = true;
    assert_eq!(f.active(), &f.night);
    f.mode = Mode::FollowTime;
    for (now, day) in [(99, false), (100, true), (200, true), (201, false)] {
        f.now = now;
        assert_eq!(f.active() == &f.day, day, "{now}");
    }
    (f.sunrise, f.sunset, f.now) = (200, 100, 150);
    assert_eq!(f.active(), &f.night);
    f.preview = Some(name("p", "q"));
    assert_eq!(f.active(), &name("p", "q"));
    assert_eq!(Mode::parse("FOLLOW_TIME"), Some(Mode::FollowTime));
    assert_eq!(Mode::parse("follow_time"), None);
}

#[test]
fn a_loaded_theme_is_kept_until_the_extensions_change() {
    let (mut session, index) = (Session::new(), index());
    let f = facts(Mode::AlwaysDay);
    let load = session.update(false, false, &f, &index);
    let path = text("stylesheets/day.json");
    assert_eq!(
        load,
        Step::Load {
            extension: 0,
            component: 0,
            path
        }
    );
    assert_eq!(session.loaded(true), Some(Step::Active(0)));
    assert_eq!(session.loaded(true), None);
    assert_eq!(session.update(false, false, &f, &index), Step::Cached(0));
    assert_eq!(session.update(false, true, &f, &index), Step::Preview);
    assert!(matches!(
        session.update(true, false, &f, &index),
        Step::Load { .. }
    ));
    assert_eq!(session.cached(), 0);
    assert_eq!(session.loaded(true), Some(Step::Active(1)));
}

#[test]
fn a_failed_load_is_tried_again_and_an_abandoned_one_is_forgotten() {
    let (mut session, index) = (Session::new(), index());
    let f = facts(Mode::AlwaysNight);
    let path = text("n.json");
    let load = Step::Load {
        extension: 0,
        component: 1,
        path,
    };
    assert_eq!(session.update(false, false, &f, &index), load);
    assert_eq!(session.loaded(false), Some(Step::Failed));
    assert_eq!(session.update(false, false, &f, &index), load);
    session.update(false, true, &f, &index);
    assert_eq!(session.loaded(true), None);
    assert_eq!(session.cached(), 0);
}

#[test]
fn a_theme_that_cannot_load_changes_nothing() {
    let (mut session, index) = (Session::new(), index());
    let mut f = facts(Mode::AlwaysDay);
    for day in [
        name("keyboard", "day"),
        name("bare", "day"),
        name("ext", "x"),
        name("none", "day"),
    ] {
        f.day = day;
        assert_eq!(session.update(false, false, &f, &index), Step::Keep);
        assert_eq!(session.loaded(true), None);
    }
}

fn row(values: &[(usize, &str)], size: usize) -> Vec<Vec<u16>> {
    let mut out = vec![vec![]; size];
    for &(at, value) in values {
        out[at] = text(value);
    }
    out
}

#[test]
fn rows_cross_both_ways() {
    let mut session = Session::new();
    let mut fields = row(
        &[
            (update::MODE, "ALWAYS_DAY"),
            (update::DAY_EXTENSION, "ext"),
            (update::DAY_COMPONENT, "day"),
            (update::SUNRISE, "0"),
            (update::SUNSET, "0"),
            (update::NOW, "0"),
            (update::KEYBOARDS, "1"),
            (update::THEMES, "1"),
        ],
        update::SIZE,
    );
    fields.extend(["kb", "ext", "1", "1", "day", "  "].map(text));
    let answer = call(&mut session, &text("update"), &fields).unwrap();
    assert_eq!(answer[step::KIND], text(&kind::LOAD.to_string()));
    assert_eq!(answer[step::PATH], text("stylesheets/day.json"));
    assert_eq!(
        (
            answer[step::EXTENSION].clone(),
            answer[step::COMPONENT].clone()
        ),
        (text("0"), text("0"))
    );
    let done = call(&mut session, &text("loaded"), &[text("1")]).unwrap();
    assert_eq!(
        (done[step::KIND].clone(), done[step::SLOT].clone()),
        (text("4"), text("0"))
    );
    fields[update::CLEAR] = text("1");
    let again = call(&mut session, &text("update"), &fields).unwrap();
    assert_eq!(
        (again[step::KIND].clone(), again[step::CLEARED].clone()),
        (text("3"), text("1"))
    );
    let cut = &fields[..fields.len() - 1];
    assert_eq!(
        call(&mut session, &text("update"), cut),
        Err(Error("an index row is cut short"))
    );
    let finished = call(&mut session, &text("loaded"), &[text("1")]).unwrap();
    assert_eq!(finished[step::SLOT], text("1"));
    assert_eq!(
        call(&mut session, &text("loaded"), &[text("1")]),
        Err(Error("no load is in progress"))
    );
    assert_eq!(
        call(&mut session, &text("other"), &[]),
        Err(Error("not an operation"))
    );
}

#[test]
fn the_base_stylesheet_compiles_and_styles_the_chips() {
    let base = rule(&text("base"), &[]).unwrap();
    assert_eq!(base, vec![text(crate::base::BASE)]);
    assert!(snygg::resolve::CompiledTheme::from_json(crate::base::BASE).is_ok());
    let palette = Palette::default();
    assert_eq!(
        chip(
            crate::base::BASE,
            &text("characters"),
            &text("unshifted"),
            &palette
        ),
        (0xFF42_4242, 0xFFFF_FFFF)
    );
    assert_eq!(
        chip("not json", &text("a"), &text("b"), &palette),
        (BACKGROUND, FOREGROUND)
    );
}

#[test]
fn dynamic_chip_colours_take_the_palette() {
    let sheet = r#"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet",
        "window":{"foreground":"dynamic-dark-color(secondary)"},
        "inline-autofill-chip":{"background":"dynamic-light-color(primary)"}}"#;
    let palette = Palette {
        names: vec![text("primary"), text("secondary")],
        light: vec![0xFF11_1111, 0xFF22_2222],
        dark: vec![0xFF33_3333, 0xFF44_4444],
    };
    assert_eq!(
        chip(sheet, &text("m"), &text("s"), &palette),
        (0xFF11_1111, 0xFF44_4444)
    );
    // A colour name the palette lacks doesn't parse, so the stylesheet styles nothing.
    let unknown = sheet.replace("light-color(primary)", "light-color(nothing)");
    let unstyled = chip(&unknown, &text("m"), &text("s"), &palette);
    assert_eq!(unstyled, (BACKGROUND, FOREGROUND));
    let mut fields = row(
        &[
            (inline::STYLESHEET, sheet),
            (inline::MODE, "m"),
            (inline::SHIFT, "s"),
        ],
        inline::SIZE,
    );
    fields.extend(
        [
            "primary",
            "secondary",
            "-15658735",
            "-14540254",
            "-13421773",
            "-12303292",
        ]
        .map(text),
    );
    assert_eq!(
        rule(&text("inline"), &fields).unwrap(),
        vec![text("-15658735"), text("-12303292")]
    );
    let accent = |specified| {
        row(
            &[
                (accent::SPECIFIED, specified),
                (accent::PREFERRED, "5"),
                (accent::SYSTEM, "7"),
            ],
            accent::SIZE,
        )
    };
    assert_eq!(
        rule(&text("primary"), &accent("1")).unwrap(),
        vec![text("5")]
    );
    assert_eq!(
        rule(&text("primary"), &accent("0")).unwrap(),
        vec![text("7")]
    );
    assert_eq!(rule(&text("other"), &[]), Err(Error("not a rule")));
}
