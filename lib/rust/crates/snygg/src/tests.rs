use crate::{
    Error,
    attributes::Attributes,
    document::{self, Configuration},
    grammar, text,
};

#[test]
fn syntax_and_lexical_ranges() {
    let r = grammar::parse(&text("key[z=1..3][a=`b`,`a`,10,2]:pressed"))
        .unwrap()
        .unwrap();
    assert_eq!(r.format(), text("key[a=10,2,`a`,`b`][z=1..3]:pressed"));
    assert!(grammar::parse(&text("key[a=01]")).unwrap().is_none());
    assert!(matches!(
        grammar::parse(&text("key[a=2147483648..2]")),
        Err(Error::Number(_))
    ));
    assert!(
        grammar::parse(&text("key[a=2147483648]"))
            .unwrap()
            .is_some()
    );
    assert!(grammar::parse(&text("@font `A\tB`")).unwrap().is_some());
}

#[test]
fn duplicate_rules_keep_first_slot_and_errors_are_deferred() {
    let keys = [
        "key[b=2][a=1]",
        "@defines",
        "key[a=1][b=2]",
        "bad:",
        "later",
    ]
    .map(text);
    let plan = document::plan(&keys, Configuration::default());
    assert_eq!(plan.len(), 4);
    assert_eq!((plan[2].source, plan[2].slot), (2, 0));
    assert!(matches!(plan[3].rule, Err(Error::InvalidRule(_))));
}

#[test]
fn schema_flags_and_original_wildcard_dots() {
    assert_eq!(
        document::schema(None, Configuration::default()),
        Err(Error::MissingSchema)
    );
    let c = Configuration {
        ignore_unsupported_schema: true,
        ..Configuration::default()
    };
    assert_eq!(
        document::schema(
            Some(&text(
                "https://schemasXflorisboardYorg/snygg/v03/stylesheet"
            )),
            c
        )
        .unwrap(),
        text("https://schemasXflorisboardYorg/snygg/v03/stylesheet")
    );
    assert!(document::schema(Some(&text(document::SCHEMA_V2)), Configuration::default()).is_ok());
}

#[test]
fn arbitrary_attribute_values_and_wrapping_ranges() {
    let a = Attributes(vec![(
        text("a"),
        vec![text("2147483647"), text("-2147483648"), vec![0xD800]],
    )]);
    assert!(!a.format().is_empty());
    assert_eq!(crate::compare(&[0xD800], &[0xD801]), -1);
}
