//! SnyggValueIcon for every kind: the Material icon each value kind drew, a shape's outline as
//! `alwaysPercentShape` makes it, a variable's link over the small icon of what `@defines` gives
//! it, where the rules list and the value row show them, and the kinds that show none.
use super::{common::*, props::*};
use keys::Rect;
use render::{
    panels::{Item, rect},
    settings_ui::{
        App,
        catalogue::theme_editor::Part,
        theme_editor::{
            glyphs::{VALUE_ICONS, icon_name, outline, shape_corners},
            paint::rule_id,
        },
    },
    text::Text,
};
use snygg::{
    text,
    value::{Corners, Encoder, ShapeKind, Unit, Value},
};

/// Every kind with an icon, on `key`, a variable for a colour, one for a shape and one for a
/// variable, and a shape of its own on `smartbar`.
const KINDS: &str = r##"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet",
    "@defines":{"--accent":"#ff000080","--edge":"rounded-corner(4dp, 10dp, 40dp, 0dp)",
        "--outer":"var(--accent)"},
    "key":{"background":"var(--accent)","foreground":"var(--nowhere)","shape":"var(--edge)",
        "shadow-color":"var(--outer)",
        "border-color":"#00ff00","font-family":"monospace","font-style":"italic",
        "font-weight":"bold","font-size":"12sp","padding":"1dp 2dp 3dp 4dp","shadow-elevation":"2dp",
        "text-align":"end","text-decoration-line":"line-through","text-overflow":"ellipsis",
        "text-max-lines":"2","clip":"yes","content-scale":"crop",
        "background-image":"uri(`flex:/images/sky.png`)","margin":"inherit"},
    "smartbar":{"shape":"cut-corner(25%, 25%, 10%, 0%)"}}"##;

/// The path Rust draws for settings icon `name`.
fn path(name: &str) -> String {
    icons::settings(name)
        .and_then(icons::path)
        .unwrap()
        .to_owned()
}
#[test]
fn every_value_icon_is_one_rust_holds() {
    for name in VALUE_ICONS {
        assert!(path(name).starts_with("M "), "{name}");
    }
}
/// The editor on [`KINDS`].
fn kinds() -> (App, Text) {
    let (mut a, mut text) = app(KINDS);
    a.scene(&mut text);
    (a, text)
}
/// The rules list's icon box for property `name` of `key`: 24 dp, centred in the slot before
/// Delete.
fn icon_box(a: &App, name: &str) -> Rect {
    icon_box_in(a, 1, name)
}
/// The rules list's icon box for property `name` of rule `r`.
fn icon_box_in(a: &App, r: usize, name: &str) -> Rect {
    let props = a.rule_properties(r);
    let n = props.iter().position(|q| q.raw == name).unwrap();
    let delete = control(a, rule_id(r, 65 + 2 * n)).unwrap().bounds;
    let d = a.config[2];
    let inset = ((48.0 * d - 24.0 * d) / 2.0).round();
    rect(
        delete.left - 48.0 * d + inset,
        delete.top + inset,
        24.0 * d,
        24.0 * d,
    )
}
/// The items the panel draws inside `b`, grown by `reach` each way.
fn items_in(a: &mut App, text: &mut Text, b: Rect, reach: f32) -> Vec<Item> {
    let panel = a.scene(text).panel.unwrap();
    let within = |i: &Item| {
        i.bounds.left >= b.left - reach
            && i.bounds.top >= b.top - reach
            && i.bounds.right <= b.right + reach
            && i.bounds.bottom <= b.bottom + reach
    };
    panel.items.into_iter().filter(within).collect()
}
fn dp(v: f32) -> Value {
    Value::Dimension(Unit::Dp, v)
}
fn word(e: Encoder, w: &str) -> Value {
    Value::Enum(e, text(w))
}

#[test]
fn each_kind_has_the_icon_compose_drew() {
    use Encoder::*;
    let cases = [
        (word(GenericFont, "monospace"), Some("font_download")),
        (Value::Font(text("Sans")), Some("font_download")),
        (word(FontStyle, "italic"), Some("format_italic")),
        (Value::FontWeight(700), Some("format_bold")),
        (Value::Padding([1.0; 4]), Some("padding")),
        (dp(2.0), Some("straighten")),
        (Value::Dimension(Unit::Sp, 12.0), Some("format_size")),
        (word(TextAlign, "left"), Some("format_align_left")),
        (word(TextAlign, "start"), Some("format_align_left")),
        (word(TextAlign, "right"), Some("format_align_right")),
        (word(TextAlign, "end"), Some("format_align_right")),
        (word(TextAlign, "justify"), Some("format_align_justify")),
        (word(TextAlign, "center"), Some("format_align_center")),
        (
            word(TextDecoration, "line-through"),
            Some("format_strikethrough"),
        ),
        (word(TextDecoration, "underline"), Some("format_underlined")),
        (word(TextDecoration, "none"), Some("format_underlined")),
        (word(TextOverflow, "ellipsis"), Some("wrap_text")),
        (
            Value::Uri(text("flex:/images/sky.png")),
            Some("attach_file"),
        ),
        (word(ContentScale, "crop"), Some("open_in_full")),
        (Value::Keyword(Yes), Some("check_box")),
        (Value::Keyword(No), Some("check_box_outline_blank")),
        (Value::MaxLines(2), None),
        (Value::Dimension(Unit::Percentage, 50.0), None),
        (Value::Keyword(Inherit), None),
        (Value::Keyword(Undefined), None),
        (Value::Variable(text("--accent")), None),
    ];
    for (v, icon) in cases {
        assert_eq!(icon_name(&v), icon, "{v:?}");
    }
}

#[test]
fn a_shape_is_outlined_as_always_percent_shape_makes_it() {
    let shape = |kind, corners| Value::Shape(kind, corners);
    // A dp corner is three times its whole dp as a percentage, at most 100; corners too big
    // for a side shrink together.
    let near = |v: &Value, size: f32, want: [f32; 4], cut: bool| {
        let (r, is_cut) = shape_corners(v, size).unwrap();
        let close = r.iter().zip(want).all(|(a, b)| (a - b).abs() < 1e-3);
        assert!(close && is_cut == cut, "{v:?}: {r:?} {is_cut}");
    };
    let rounded = shape(ShapeKind::Rounded, Corners::Dp([4.7, 10.0, 40.0, 0.0]));
    let fit = 100.0 / 130.0;
    near(
        &rounded,
        100.0,
        [12.0 * fit, 30.0 * fit, 100.0 * fit, 0.0],
        false,
    );
    let cut_percent = shape(ShapeKind::Cut, Corners::Percent([10, 20, 30, 40]));
    near(&cut_percent, 50.0, [5.0, 10.0, 15.0, 20.0], true);
    near(
        &shape(ShapeKind::Circle, Corners::Dp([0.0; 4])),
        58.0,
        [29.0; 4],
        false,
    );
    near(
        &shape(ShapeKind::Rectangle, Corners::Dp([8.0; 4])),
        58.0,
        [0.0; 4],
        false,
    );
    assert_eq!(shape_corners(&dp(4.0), 58.0), None);
    // The outline: a 2 px line inside the edge, clear within, the corners cut or rounded away.
    let ink = 0xff11_2233_u32 as i32;
    let at = |px: &[i32], x: usize, y: usize| px[y * 20 + x];
    let boxed = outline(20, ([0.0; 4], false), 2.0, 0xff11_2233);
    assert_eq!(
        [at(&boxed, 0, 0), at(&boxed, 1, 10), at(&boxed, 19, 10)],
        [ink; 3]
    );
    assert_eq!([at(&boxed, 2, 10), at(&boxed, 10, 10)], [0x0011_2233; 2]);
    let round = outline(20, ([10.0; 4], false), 2.0, 0xff11_2233);
    assert_eq!((at(&round, 0, 0), at(&round, 10, 0)), (0x0011_2233, ink));
    let cut = outline(20, ([6.0; 4], true), 2.0, 0xff11_2233);
    assert_eq!(
        (at(&cut, 1, 1), at(&cut, 0, 10), at(&cut, 3, 3)),
        (0x0011_2233, ink, ink)
    );
    assert_eq!(at(&cut, 5, 5), 0x0011_2233, "clear inside the cut's line");
    assert_eq!(
        at(&cut, 3, 4),
        ink,
        "the cut's line as thick across the corner"
    );
}

#[test]
fn the_rules_list_shows_each_kinds_icon_before_delete() {
    let (mut a, mut text) = kinds();
    let d = a.config[2];
    for (name, icon) in [
        ("font-family", "font_download"),
        ("font-style", "format_italic"),
        ("font-weight", "format_bold"),
        ("font-size", "format_size"),
        ("padding", "padding"),
        ("shadow-elevation", "straighten"),
        ("text-align", "format_align_right"),
        ("text-decoration-line", "format_strikethrough"),
        ("text-overflow", "wrap_text"),
        ("clip", "check_box"),
        ("content-scale", "open_in_full"),
        ("background-image", "attach_file"),
        ("foreground", "link"),
    ] {
        let b = icon_box(&a, name);
        let drawn = items_in(&mut a, &mut text, b, 0.0);
        let found = drawn.iter().find(|i| i.icon == path(icon));
        let i = found.unwrap_or_else(|| panic!("{name} has no {icon}"));
        assert_eq!(
            (i.bounds, i.icon_style.foreground),
            (b, a.colors[2]),
            "{name}"
        );
    }
    // A static colour's swatch; the line count and inherit show nothing, and their value keeps
    // the slot.
    let border = icon_box(&a, "border-color");
    let swatch = items_in(&mut a, &mut text, border, 0.0);
    assert!(swatch.iter().any(|i| i.styles[0].background == 0xff00_ff00));
    for name in ["text-max-lines", "margin"] {
        let b = icon_box(&a, name);
        assert!(items_in(&mut a, &mut text, b, 4.0 * d).is_empty(), "{name}");
    }
    // smartbar's own shape: its 22 dp outline, centred in the box.
    assert_eq!(a.rule_properties(2)[0].raw, "shape");
    let b = icon_box_in(&a, 2, "shape");
    let n = (22.0 * d).round();
    let inset = ((b.width() - n) / 2.0).round();
    let outlined = rect(b.left + inset, b.top + inset, n, n);
    let drawn = items_in(&mut a, &mut text, b, 0.0);
    assert!(
        drawn
            .iter()
            .any(|i| i.media.is_some() && i.bounds == outlined)
    );
}

#[test]
fn a_variable_shows_its_link_over_the_small_icon_of_its_value() {
    let (mut a, mut text) = kinds();
    let d = a.config[2];
    let b = icon_box(&a, "background");
    let (x, y) = (b.left, b.top - 2.0 * d);
    let drawn = items_in(&mut a, &mut text, b, 4.0 * d);
    let link = drawn.iter().find(|i| i.icon == path("link")).unwrap();
    assert_eq!(link.bounds, rect(x, y, 16.0 * d, 16.0 * d));
    // The translucent accent's small swatch 8 dp in, over its checkered backing, and the
    // link's backing between them.
    let small = rect(x + 8.0 * d, y + 8.0 * d, 16.0 * d, 16.0 * d);
    let swatch = drawn
        .iter()
        .position(|i| i.styles[0].background == 0x80ff_0000);
    let swatch = swatch.expect("the accent's small swatch");
    assert_eq!(drawn[swatch].bounds, small);
    assert!(drawn.iter().any(|i| i.media.is_some() && i.bounds == small));
    let backing = rect(x + d, y + 2.0 * d, 16.0 * d, 12.0 * d);
    let under = drawn
        .iter()
        .position(|i| i.bounds == backing)
        .expect("the link's backing");
    let over = drawn.iter().position(|i| i.icon == path("link")).unwrap();
    assert!(swatch < under && under < over, "swatch, backing, then link");
    // The shape variable's small outline: 16 dp, centred on the small icon's box.
    let edge = icon_box(&a, "shape");
    let (x, y) = (edge.left, edge.top - 2.0 * d);
    let drawn = items_in(&mut a, &mut text, edge, 4.0 * d);
    let n = (16.0 * d).round();
    let outlined = rect(x + 8.0 * d, y + 8.0 * d, n, n);
    assert!(
        drawn
            .iter()
            .any(|i| i.media.is_some() && i.bounds == outlined)
    );
}

#[test]
fn the_value_row_shows_a_variables_link_even_before_it_names_one() {
    let (mut a, mut text) = kinds();
    named(&mut a);
    add_to(&mut a, &mut text, 1);
    pick(&mut a, &mut text, Part::PropName, "Border width");
    pick(&mut a, &mut text, Part::PropEncoder, "Variable");
    let d = a.config[2];
    let row = control(&a, part_id(&a, Part::PropValue)).unwrap().bounds;
    let size = 24.0 * d;
    let left = a.config[0] - 16.0 * d - size;
    let b = rect(
        left,
        row.top + ((row.height() - size) / 2.0).round(),
        size,
        size,
    );
    let drawn = items_in(&mut a, &mut text, b, 0.0);
    assert!(
        drawn
            .iter()
            .any(|i| i.icon == path("link") && i.bounds == b)
    );
    // --edge: the shape's small outline under the link.
    pick(&mut a, &mut text, Part::PropValue, "--edge");
    let drawn = items_in(&mut a, &mut text, b, 4.0 * d);
    assert!(
        drawn
            .iter()
            .any(|i| i.icon == path("link") && i.bounds.width() == 16.0 * d)
    );
    assert!(
        drawn.iter().any(|i| i.media.is_some()),
        "the shape's outline"
    );
}

#[test]
fn a_variable_naming_a_variable_shows_two_links_and_a_loop_ends() {
    let (mut a, mut text) = kinds();
    let d = a.config[2];
    let b = icon_box(&a, "shadow-color");
    let (x, y, s) = (b.left, b.top - 2.0 * d, 16.0 * d);
    // --outer names --accent: the link over --accent's small icon, which is its link alone.
    let panel = a.scene(&mut text).panel.unwrap();
    let near = |i: &&Item| {
        let (left, top) = (i.bounds.left, i.bounds.top);
        left >= b.left - 4.0 * d && left < b.right && top >= b.top - 4.0 * d && top < b.bottom
    };
    let drawn: Vec<&Item> = panel.items.iter().filter(near).collect();
    let links: Vec<Rect> = drawn
        .iter()
        .filter(|i| i.icon == path("link"))
        .map(|i| i.bounds)
        .collect();
    let small = rect(x + 8.0 * d, y + 8.0 * d, s, s);
    assert_eq!(links, [small, rect(x, y, s, s)]);
    assert!(
        !drawn.iter().any(|i| i.styles[0].background == 0x80ff_0000),
        "no swatch at the small form's own small size"
    );
    // A variable naming itself, which Compose followed without end, stops at its small link.
    let sheet = r#"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet",
        "@defines":{"--loop":"var(--loop)"},"key":{"background":"var(--loop)"}}"#;
    let (mut a, mut text) = app(sheet);
    assert_eq!(a.rule_properties(1)[0].raw, "background");
    let panel = a.scene(&mut text).panel.unwrap();
    assert!(panel.items.iter().any(|i| i.icon == path("link")));
}
