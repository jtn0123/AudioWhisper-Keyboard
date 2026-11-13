//! Frozen independent Kotlin reference, verified again by the JVM match suite.
use render::{
    scene::{Key, Scene},
    style::{Style, color},
    theme::{Request, Theme},
};
use snygg::{text, value::Value};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..")
}
fn request(q: &serde_json::Value) -> Request {
    let s = |i: usize| q[i].as_str().unwrap();
    Request {
        element: s(0).into(),
        mode: s(1).into(),
        shift: s(2).into(),
        code: s(3).parse().unwrap(),
        key_type: s(4).into(),
        selector: s(5).parse().unwrap(),
        auto_commit: s(6) == "1",
        popup_source: None,
    }
}
#[test]
fn every_bundled_theme_resolves_renderer_keys_candidates_and_popups_like_kotlin() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!("themes.json")).unwrap();
    let names = fixture["names"].as_array().unwrap();
    let colors = fixture["colors"].as_array().unwrap();
    let palette = |offset: usize| {
        names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                (
                    text(name.as_str().unwrap()),
                    color(colors[offset + i].as_i64().unwrap() as u32),
                )
            })
            .collect()
    };
    let themes = fixture["themes"].as_object().unwrap();
    assert_eq!(themes.len(), 12);
    for (path, cases) in themes {
        let json = std::fs::read_to_string(root().join(path)).unwrap();
        let mut theme =
            Theme::new(&json, palette(0), palette(names.len()), 2.625, 1.1, 1.0).unwrap();
        for case in cases.as_array().unwrap() {
            let request = request(&case["query"]);
            let properties = theme.properties(&request);
            let actual: serde_json::Map<_, _> = properties
                .iter()
                .map(|(k, v)| {
                    (
                        String::from_utf16(k).unwrap(),
                        serde_json::Value::String(
                            String::from_utf16(&v.encoder().serialize(v).unwrap()).unwrap(),
                        ),
                    )
                })
                .collect();
            assert_eq!(
                serde_json::Value::Object(actual),
                case["expected"],
                "{path} {request:?}"
            );
            // Exercise scene materialization too: it must choose the same state and scale.
            let expected = Style::from_properties(&properties, 2.625, 1.1);
            assert_eq!(theme.style(&request), expected);
            let mut scene = Scene {
                keys: vec![Key {
                    code: request.code,
                    bounds: Default::default(),
                    label: String::new(),
                    hint: String::new(),
                    icon: String::new(),
                    enabled: request.selector != 4,
                    styles: Default::default(),
                    hints: Default::default(),
                }],
                candidates: vec![String::new()],
                ..Scene::default()
            };
            theme.apply(
                &mut scene,
                &request.mode,
                &request.shift,
                std::slice::from_ref(&request.key_type),
                &[i32::from(request.auto_commit)],
            );
            let state = usize::from(request.selector == 1);
            match request.element.as_str() {
                "key" if request.selector != 2 => assert_eq!(scene.keys[0].styles[state], expected),
                "key-hint" if request.selector != 2 => {
                    assert_eq!(scene.keys[0].hints[state], expected)
                }
                "smartbar-candidate-word" if request.selector != 2 => {
                    assert_eq!(scene.candidate_styles[0][state].container, expected)
                }
                "smartbar-candidate-word-text" if request.selector != 2 => {
                    assert_eq!(scene.candidate_styles[0][state].text, expected)
                }
                "key-popup-box" | "key-popup-element" => {
                    theme.popup_styles(&mut scene, &request, &[request.code]);
                    if request.element == "key-popup-box" && request.selector == 0 {
                        assert_eq!(scene.popup_style, expected);
                    }
                    if request.element == "key-popup-element" && request.selector != 1 {
                        assert_eq!(
                            scene.popup_styles[0][usize::from(request.selector == 2)],
                            expected
                        );
                    }
                }
                _ => {}
            }
        }
        let before = theme.cached_styles();
        for case in cases.as_array().unwrap() {
            theme.style(&request(&case["query"]));
        }
        assert_eq!(
            before,
            theme.cached_styles(),
            "repeat queries must hit cached styles"
        );
    }
}
#[test]
fn extension_source_replacement_and_stateful_geometry_use_resolved_styles() {
    let json = r##"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","window":{"background":"#123456"},"key":{"font-size":"20sp","shape":"rounded-corner(20%,30%,40%,10%)"},"key[type=`function`]:pressed":{"background":"#abcdef","font-size":"30sp"},"key[code=-11][shiftstate=`caps_lock`]":{"foreground":"#00ff00"},"key-hint":{"font-size":"10sp"},"key-popup-element:focus":{"foreground":"#ff0000"}}"##;
    let mut theme = Theme::new(json, vec![], vec![], 2.0, 1.5, 1.2).unwrap();
    let r = Request {
        element: "key".into(),
        mode: "characters".into(),
        shift: "caps_lock".into(),
        code: -11,
        key_type: "function".into(),
        selector: 1,
        auto_commit: false,
        popup_source: None,
    };
    let style = theme.style(&r);
    assert_eq!(style.background, 0xffabcdef);
    assert_eq!(style.foreground, 0xff00ff00);
    assert!((style.size - 108.0).abs() < 0.001);
    assert_eq!(style.radii(100.0, 50.0), [10.0, 15.000001, 20.0, 5.0]);
    let next = json.replace("#abcdef", "#fedcba");
    let mut next = Theme::new(&next, vec![], vec![], 2.0, 1.5, 1.2).unwrap();
    assert_eq!(next.style(&r).background, 0xfffedcba);
    assert_ne!(next.style(&r), style);
    assert_eq!(
        theme
            .properties(&r.element("key-hint", 1))
            .iter()
            .find(|(k, _)| *k == text("font-size"))
            .unwrap()
            .1,
        Value::Dimension(snygg::value::Unit::Sp, 12.0)
    );
}

#[test]
fn unsupported_families_fall_back_but_weights_select_real_faces() {
    use snygg::value::Encoder;
    let mono = (
        text("font-family"),
        Value::Enum(Encoder::GenericFont, text("monospace")),
    );
    assert!(Style::from_properties(&vec![mono.clone()], 1.0, 1.0).mono);
    for family in [
        Value::Font(text("Extension font")),
        Value::Enum(Encoder::GenericFont, text("serif")),
    ] {
        assert!(!Style::from_properties(&vec![(text("font-family"), family)], 1.0, 1.0).mono);
    }
    let bold = Style::from_properties(
        &vec![mono, (text("font-weight"), Value::FontWeight(700))],
        1.0,
        1.0,
    );
    assert!(bold.mono);
    assert_eq!(bold.weight, 700);
}

#[test]
fn popup_choice_inherits_from_its_originating_key_box_and_cache_context() {
    let json = r##"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","key-popup-box[code=97]":{"font-size":"30sp","foreground":"#123456"},"key-popup-box[code=101]":{"font-size":"18sp","foreground":"#abcdef"}}"##;
    let mut theme = Theme::new(json, vec![], vec![], 1.0, 1.0, 1.0).unwrap();
    let mut scene = Scene::default();
    let mut source = Request {
        element: String::new(),
        mode: "characters".into(),
        shift: "unshifted".into(),
        code: 97,
        key_type: "character".into(),
        selector: 0,
        auto_commit: false,
        popup_source: None,
    };
    for (code, size, foreground) in [
        (97, 30.0, 0xff123456),
        (101, 18.0, 0xffabcdef),
        (97, 30.0, 0xff123456),
    ] {
        source.code = code;
        theme.popup_styles(&mut scene, &source, &[233]);
        assert_eq!(scene.popup_style.size, size);
        for style in &scene.popup_styles[0] {
            assert_eq!(
                style.size, size,
                "choice inherits the source box, not its own code's box"
            );
            assert_eq!(style.foreground, foreground);
        }
    }
}
