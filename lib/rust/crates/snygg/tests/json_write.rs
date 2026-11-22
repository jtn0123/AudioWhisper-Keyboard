//! The theme editor's stylesheet serializer: kotlinx's pretty print, exactly. The JVM match
//! test (`SettingsThemeEditorMatchTest`) holds it byte for byte against `SnyggStylesheet.toJson`;
//! these hold its shape and that every shipped stylesheet survives a round trip.
use snygg::{
    document::SCHEMA_V2,
    json::parse,
    json_write::to_pretty_json,
    resolve::{PropertySets, Stylesheet},
    text,
};
use std::path::PathBuf;

fn shipped() -> Vec<(PathBuf, String)> {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../app/src/main/assets/ime/theme");
    let mut out = vec![];
    for ext in std::fs::read_dir(root).unwrap() {
        let dir = ext.unwrap().path().join("stylesheets");
        for sheet in std::fs::read_dir(dir).unwrap() {
            let path = sheet.unwrap().path();
            let text = std::fs::read_to_string(&path).unwrap();
            out.push((path, text));
        }
    }
    out.sort();
    out
}

fn write(json: &str) -> String {
    let (schema, sheet) = parse(json, false).unwrap();
    to_pretty_json(&schema, &sheet).unwrap()
}

#[test]
fn every_shipped_stylesheet_round_trips_to_the_same_rules() {
    let sheets = shipped();
    assert_eq!(sheets.len(), 12);
    for (path, json) in sheets {
        let written = write(&json);
        assert_eq!(
            parse(&written, false).unwrap(),
            parse(&json, false).unwrap(),
            "{path:?}"
        );
        assert_eq!(write(&written), written, "{path:?}: writing is stable");
    }
}

#[test]
fn the_layout_is_kotlinx_pretty_print_with_a_two_space_indent() {
    // Values come out in their encoders' canonical forms, as Kotlin's `serialize` writes them.
    let json = r##"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","@defines":{},
        "key:pressed":{"foreground":"#ff0000","shape":"rounded-corner(8dp, 8dp, 8dp, 8dp)"},
        "@font `A b`":[{"src":"uri(`flex:/a.ttf`)"},{"src":"uri(`flex:/b.ttf`)","font-weight":"bold"}],
        "@font `Empty`":[]}"##;
    let expected = [
        "{",
        r##"  "$schema": "https://schemas.florisboard.org/snygg/v2/stylesheet","##,
        r##"  "@defines": {},"##,
        r##"  "key:pressed": {"##,
        r##"    "foreground": "rgba(255,0,0,1)","##,
        r##"    "shape": "rounded-corner(8dp,8dp,8dp,8dp)""##,
        "  },",
        r##"  "@font `A b`": ["##,
        "    {",
        r##"      "src": "uri(`flex:/a.ttf`)""##,
        "    },",
        "    {",
        r##"      "src": "uri(`flex:/b.ttf`)","##,
        r##"      "font-weight": "bold""##,
        "    }",
        "  ],",
        r##"  "@font `Empty`": []"##,
        "}",
    ]
    .join("\n");
    assert_eq!(write(json), expected);
}

#[test]
fn strings_escape_only_what_kotlinx_escapes() {
    let mut sheet = Stylesheet { rules: vec![] };
    sheet
        .rules
        .push((snygg::grammar::Rule::Defines, PropertySets::Single(vec![])));
    let schema = text("q\"b\\s/\n\t\r\u{8}\u{c}\u{1}\u{1f}é😀\u{7f}");
    let out = to_pretty_json(&schema, &sheet).unwrap();
    assert!(
        out.contains(r##""q\"b\\s/\n\t\r\b\f\u0001\u001fé😀"##),
        "{out}"
    );
    assert!(out.contains("\u{7f}\""), "DEL is not escaped");
}

#[test]
fn a_font_set_without_its_src_fails_as_to_json_does() {
    let mut sheet = Stylesheet { rules: vec![] };
    sheet.rules.push((
        snygg::grammar::Rule::Font(text("A")),
        PropertySets::Multiple(vec![vec![]]),
    ));
    let error = to_pretty_json(&text(SCHEMA_V2), &sheet).unwrap_err();
    assert_eq!(
        error.0,
        "Rule '@font `A`' is missing required property 'src'"
    );
}

#[test]
fn lenient_parsing_keeps_what_it_can_and_strict_parsing_refuses() {
    let json = r##"{"$schema":"wrong","key":{"foreground":"#ff0000","nope":"x"},"bad:":{}}"##;
    assert!(parse(json, false).is_err());
    let (schema, sheet) = parse(json, true).unwrap();
    assert_eq!(schema, text(SCHEMA_V2));
    assert_eq!(sheet.rules.len(), 1);
    let PropertySets::Single(props) = &sheet.rules[0].1 else {
        panic!("a single set")
    };
    assert_eq!(props.len(), 1);
}

/// Any JSON as kotlinx pretty prints it: an element or key per line two spaces deeper, empty
/// collections closed on their own line, keys in insertion order and kotlinx's escapes. The JVM
/// match test (`SettingsExtensionEditorMatchTest`) holds an extension's manifest byte for byte
/// against `ExtensionJsonConfig`.
#[test]
fn pretty_prints_any_json_as_kotlinx_does() {
    let value = serde_json::json!({
        "$": "ime.extension.theme",
        "meta": {"id": "a.b", "keywords": ["x", "y \"q\"\n\u{1}"], "empty": []},
        "none": {},
        "night": false,
        "n": 12,
        "z": null,
    });
    let expected = "{\n  \"$\": \"ime.extension.theme\",\n  \"meta\": {\n    \"id\": \"a.b\",\n    \
                    \"keywords\": [\n      \"x\",\n      \"y \\\"q\\\"\\n\\u0001\"\n    ],\n    \
                    \"empty\": []\n  },\n  \"none\": {},\n  \"night\": false,\n  \"n\": 12,\n  \
                    \"z\": null\n}";
    assert_eq!(snygg::json_write::pretty(&value), expected);
    assert_eq!(snygg::json_write::pretty(&serde_json::json!([])), "[]");
    assert_eq!(
        snygg::json_write::pretty(&serde_json::json!("é\\")),
        "\"é\\\\\""
    );
}

/// A stylesheet written by [`to_pretty_json`] reads back through `serde_json` and prints the same
/// through [`pretty`](snygg::json_write::pretty): the two writers agree wherever both apply.
#[test]
fn pretty_agrees_with_the_stylesheet_writer() {
    for (path, json) in shipped() {
        let written = write(&json);
        let value: serde_json::Value = serde_json::from_str(&written).unwrap();
        assert_eq!(
            snygg::json_write::pretty(&value),
            written,
            "{}",
            path.display()
        );
    }
}
