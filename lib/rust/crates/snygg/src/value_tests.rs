use crate::{
    properties::{self, Configuration, PropertyError, PropertyInput},
    text,
    value::{Corners, Encoder, ShapeKind, Unit, Value},
    value_spec::Failure,
};
#[test]
fn renderer_consumes_plain_values_without_a_vm() {
    assert_eq!(
        Encoder::Dp.deserialize(&text(".5dp")),
        Ok(Value::Dimension(Unit::Dp, 0.5))
    );
    assert_eq!(
        Encoder::DynamicDark.deserialize(&text("dynamic-dark-color(primary)")),
        Ok(Value::DynamicColor {
            dark: true,
            name: text("primary")
        })
    );
    assert_eq!(
        Encoder::RoundedDp.deserialize(&text("rounded-corner(1.5dp,2dp,3dp,4dp)")),
        Ok(Value::Shape(
            ShapeKind::Rounded,
            Corners::Dp([1.5, 2.0, 3.0, 4.0])
        ))
    );
    assert_eq!(
        Encoder::Variable.deserialize(&text("var(--missing)")),
        Ok(Value::Variable(text("--missing")))
    );
}
#[test]
fn historical_grammar_quirks_remain() {
    assert!(matches!(
        Encoder::CutDp.deserialize(&text("cut-corner(1.5dp,2dp,3dp,4dp)")),
        Err(Failure::Number(_))
    ));
    assert!(Encoder::Sp.deserialize(&text("0.5sp")).is_err());
    assert!(
        Encoder::Color
            .deserialize(&text("rgba(1,2,3,1.00)"))
            .is_err()
    );
    assert_eq!(
        Encoder::FontWeight.serialize(&Value::FontWeight(400)),
        Ok(text("normal"))
    );
    assert_eq!(
        Encoder::Padding.serialize(&Encoder::Padding.deserialize(&text("2dp")).unwrap()),
        Ok(text("2dp 2dp 2dp 2dp"))
    );
}
#[test]
fn property_errors_follow_source_order_and_leniency() {
    let entries = vec![
        PropertyInput {
            name: text("unknown"),
            encoders: None,
            input: Err(()),
        },
        PropertyInput {
            name: text("src"),
            encoders: Some(vec![Encoder::Uri]),
            input: Err(()),
        },
    ];
    assert_eq!(
        properties::decode(&entries, &[text("src")], &Configuration::default()),
        Err(PropertyError::InvalidProperty(0))
    );
    assert_eq!(
        properties::decode(
            &entries,
            &[text("src")],
            &Configuration {
                ignore_properties: true,
                ignore_values: true
            }
        ),
        Err(PropertyError::Json(1))
    );
    assert_eq!(
        properties::decode(&[], &[text("src")], &Configuration::default()),
        Err(PropertyError::Missing(text("src")))
    );
}
#[test]
fn hostile_packets_and_text_are_not_panics() {
    assert!(Encoder::from_id(usize::MAX).is_none());
    for e in crate::value::ENCODERS {
        for s in [
            "",
            "?",
            "9dp",
            "uri(`flex://[::1]`)",
            "99999999999999999999999999999999",
        ] {
            let _ = e.deserialize(&text(s));
        }
    }
}
