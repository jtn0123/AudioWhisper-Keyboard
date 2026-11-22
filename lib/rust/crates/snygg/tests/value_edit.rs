//! The values the theme editor's property screen edits: every encoder's starting value (what
//! choosing it in the encoder dropdown sets) writes and reads back as itself, the three that
//! start empty refuse to be written until a name is chosen, and the text the screen's fields
//! hand the encoders reads as the value Compose's dialog built.
use snygg::{
    text,
    value::{Corners, ENCODERS, Encoder, ShapeKind, Unit, Value},
};

fn written(e: Encoder, v: &Value) -> Option<String> {
    e.serialize(v).ok().map(|t| String::from_utf16_lossy(&t))
}

#[test]
fn every_encoders_default_value_writes_and_reads_back_as_itself() {
    let mut forms = vec![];
    for e in ENCODERS {
        let start = e.default_value();
        assert_eq!(start.encoder(), e, "{e:?}");
        let Some(form) = written(e, &start) else {
            continue;
        };
        assert_eq!(e.deserialize(&text(&form)), Ok(start), "{e:?}");
        forms.push(form);
    }
    assert_eq!(
        forms,
        [
            "inherit",
            "yes",
            "no",
            "0dp",
            "24sp",
            "0%",
            "rgba(0,0,0,1)",
            "dynamic-light-color(primary)",
            "dynamic-dark-color(primary)",
            "system",
            "normal",
            "normal",
            "0dp 0dp 0dp 0dp",
            "rectangle()",
            "circle()",
            "cut-corner(0dp,0dp,0dp,0dp)",
            "cut-corner(0%,0%,0%,0%)",
            "rounded-corner(0dp,0dp,0dp,0dp)",
            "rounded-corner(0%,0%,0%,0%)",
            "left",
            "none",
            "none",
            "clip",
            "fill-bounds",
        ]
    );
}

#[test]
fn undefined_and_the_values_that_start_without_a_name_are_not_written() {
    // The screen marks these invalid until a font, a file or a variable is chosen.
    for e in [
        Encoder::Undefined,
        Encoder::CustomFont,
        Encoder::Uri,
        Encoder::Variable,
    ] {
        assert_eq!(written(e, &e.default_value()), None, "{e:?}");
    }
}

#[test]
fn chosen_names_write_as_compose_wrote_them_and_read_back() {
    let cases = [
        (Value::Font(text("Sans")), "`Sans`"),
        (Value::Variable(text("--accent")), "var(--accent)"),
        (
            Value::Uri(text("flex:/fonts/a%20b.ttf")),
            "uri(`flex:/fonts/a%20b.ttf`)",
        ),
        (
            Value::DynamicColor {
                dark: true,
                name: text("onSurface"),
            },
            "dynamic-dark-color(onSurface)",
        ),
        (Value::Enum(Encoder::TextAlign, text("end")), "end"),
    ];
    for (v, form) in cases {
        let e = v.encoder();
        assert_eq!(written(e, &v).as_deref(), Some(form), "{e:?}");
        assert_eq!(e.deserialize(&text(form)), Ok(v));
    }
}

#[test]
fn the_colour_field_takes_hex_and_rgb_forms_and_refuses_names() {
    let red = Value::Color([1.0, 0.0, 0.0, 1.0]);
    for form in ["#ff0000", "#FF0000", "rgb(255,0,0)", "rgba(255,0,0,1)"] {
        assert_eq!(
            Encoder::Color.deserialize(&text(form)),
            Ok(red.clone()),
            "{form}"
        );
    }
    // Alpha 0x80 is 128/255, written as Java's Float.toString writes it.
    let half = Encoder::Color.deserialize(&text("#ff000080")).unwrap();
    assert_eq!(
        written(Encoder::Color, &half).unwrap(),
        "rgba(255,0,0,0.5019608)"
    );
    for form in ["red", "#ff00", "rgb(256,0,0)", ""] {
        assert!(Encoder::Color.deserialize(&text(form)).is_err(), "{form}");
    }
}

#[test]
fn the_line_counts_and_weights_the_screen_offers_read_back() {
    assert_eq!(
        Encoder::MaxLines.deserialize(&text("none")),
        Ok(Value::MaxLines(i32::MAX))
    );
    assert_eq!(
        Encoder::MaxLines.deserialize(&text("3")),
        Ok(Value::MaxLines(3))
    );
    for refused in ["0", "-1", "2.5"] {
        assert!(Encoder::MaxLines.deserialize(&text(refused)).is_err());
    }
    // The weight dropdown lists the nine words; each is its hundred.
    let words = &Encoder::FontWeight.keywords()[..9];
    assert_eq!(words[0], "thin");
    assert_eq!(words[8], "black");
    for (k, word) in words.iter().enumerate() {
        let v = Value::FontWeight(100 * (k as i32 + 1));
        assert_eq!(Encoder::FontWeight.deserialize(&text(word)), Ok(v.clone()));
        assert_eq!(written(Encoder::FontWeight, &v).as_deref(), Some(*word));
    }
}

#[test]
fn sizes_sides_and_corners_write_in_their_units() {
    let cases = [
        (Value::Dimension(Unit::Dp, 12.5), "12.5dp"),
        (Value::Dimension(Unit::Sp, 14.0), "14sp"),
        (Value::Dimension(Unit::Percentage, 0.5), "50%"),
        (Value::Padding([1.0, 2.5, 3.0, 4.0]), "1dp 2.5dp 3dp 4dp"),
        (
            Value::Shape(ShapeKind::Rounded, Corners::Dp([1.0, 2.5, 3.0, 4.0])),
            "rounded-corner(1dp,2.5dp,3dp,4dp)",
        ),
        (
            Value::Shape(ShapeKind::Cut, Corners::Percent([10, 20, 30, 40])),
            "cut-corner(10%,20%,30%,40%)",
        ),
    ];
    for (v, form) in cases {
        let e = v.encoder();
        assert_eq!(written(e, &v).as_deref(), Some(form), "{e:?}");
        assert_eq!(e.deserialize(&text(form)), Ok(v));
    }
    // The bounds the fields check: dp from 0, sp from 1.
    let negative = Value::Dimension(Unit::Dp, -1.0);
    assert_eq!(written(Encoder::Dp, &negative), None);
    assert_eq!(written(Encoder::Sp, &Value::Dimension(Unit::Sp, 0.5)), None);
}

#[test]
fn a_cut_corner_in_fractional_dp_is_written_but_does_not_read_back() {
    // Why the screen asks that a value read back, not only that it be written.
    let cut = Value::Shape(ShapeKind::Cut, Corners::Dp([2.5, 0.0, 0.0, 0.0]));
    let form = written(Encoder::CutDp, &cut).unwrap();
    assert_eq!(form, "cut-corner(2.5dp,0dp,0dp,0dp)");
    assert!(Encoder::CutDp.deserialize(&text(&form)).is_err());
}
