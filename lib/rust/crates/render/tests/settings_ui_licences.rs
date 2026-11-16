//! ThirdPartyLicensesScreen drawn natively: AboutLibraries' parse of the generated JSON (its
//! licence sets, author line and order), Html.fromHtml's text for the dialog, the list drawing
//! only the rows on screen, its lazy scrollbar, and the licence dialog.
use render::{
    settings_ui::{
        Action, App,
        licences::{BODY_ID, LIBRARIES, ROUTE, ROW_ID, data, html},
    },
    text::Text,
};

fn library(name: &str, licences: &[&str], extra: &str) -> String {
    let ids: Vec<String> = licences.iter().map(|l| format!("\"{l}\"")).collect();
    format!(
        r#"{{"uniqueId":"g:{name}","name":"{name}","licenses":[{}]{extra}}}"#,
        ids.join(",")
    )
}
fn json(libraries: &[String], licences: &[&str]) -> String {
    let licences: Vec<String> = licences
        .iter()
        .map(|l| format!(r#""{l}":{{"name":"{l} name","content":"{l} text"}}"#))
        .collect();
    format!(
        r#"{{"libraries":[{}],"licenses":{{{}}}}}"#,
        libraries.join(","),
        licences.join(",")
    )
}
fn names(l: &data::Libraries) -> Vec<&str> {
    l.libraries.iter().map(|l| l.name.as_str()).collect()
}
fn sets(l: &data::Libraries) -> Vec<Vec<&str>> {
    l.libraries
        .iter()
        .map(|b| {
            b.licences
                .iter()
                .map(|&i| l.licences[i].hash.as_str())
                .collect()
        })
        .collect()
}

#[test]
fn licence_sets_iterate_as_a_kotlin_hash_set_does() {
    // Each expectation is what `HashSet(mapCapacity(n))` iterates in a JVM, after `addAll`.
    let cases: [(&[&str], &[&str]); 5] = [
        (&["MIT", "Apache-2.0"], &["Apache-2.0", "MIT"]),
        (
            &["Apache-2.0", "MIT", "icu4c"],
            &["Apache-2.0", "MIT", "icu4c"],
        ),
        (
            &["icu4c", "MIT", "Apache-2.0", "BSD-3-Clause", "GPL-2.0"],
            &["Apache-2.0", "GPL-2.0", "MIT", "icu4c", "BSD-3-Clause"],
        ),
        (&["b", "a", "MIT", "b"], &["a", "MIT", "b"]),
        (
            &[
                "EPL-1.0",
                "LGPL-2.1",
                "MPL-2.0",
                "CC0-1.0",
                "Unlicense",
                "ISC",
                "Zlib",
                "BSD-2-Clause",
                "Apache-2.0",
                "MIT",
                "icu4c",
                "asdkl-1",
            ],
            &[
                "Zlib",
                "Unlicense",
                "LGPL-2.1",
                "BSD-2-Clause",
                "EPL-1.0",
                "MPL-2.0",
                "ISC",
                "Apache-2.0",
                "CC0-1.0",
                "MIT",
                "asdkl-1",
                "icu4c",
            ],
        ),
    ];
    for (given, expected) in cases {
        let mut known: Vec<&str> = given.to_vec();
        known.sort_unstable();
        known.dedup();
        let parsed = data::parse(&json(&[library("x", given, "")], &known));
        assert_eq!(sets(&parsed), [expected.to_vec()], "{given:?}");
    }
}

#[test]
fn unknown_licence_keys_drop_before_the_set_is_built() {
    let parsed = data::parse(&json(&[library("x", &["MIT", "nope"], "")], &["MIT"]));
    assert_eq!(sets(&parsed), [["MIT"]]);
}

#[test]
fn libraries_sort_by_lowercased_name_stably_in_utf16_order() {
    // "𝒜" is a surrogate pair (D835 ...), below fullwidth "ａ" (FF41) in UTF-16 order though
    // above it in code points; "b" and "B" keep their file order.
    let given = ["b", "\u{FF21}", "𝒜", "B", "Ä", "a"];
    let libraries: Vec<String> = given.iter().map(|n| library(n, &[], "")).collect();
    let parsed = data::parse(&json(&libraries, &[]));
    assert_eq!(names(&parsed), ["a", "b", "B", "Ä", "𝒜", "\u{FF21}"]);
}

#[test]
fn the_author_is_the_developers_else_the_organisation() {
    let libraries = [
        library(
            "a",
            &[],
            r#","developers":[{"name":"One"},{"name":null},{"name":"Two"}]"#,
        ),
        library(
            "b",
            &[],
            r#","developers":[],"organization":{"name":"Org"}"#,
        ),
        library("c", &[], r#","organization":{"url":"x"}"#),
        library("d", &[], r#","organization":{"name":null}"#),
        library("e", &[], ""),
    ];
    let parsed = data::parse(&json(&libraries, &[]));
    let authors: Vec<&str> = parsed.libraries.iter().map(|l| l.author.as_str()).collect();
    assert_eq!(authors, ["One, Two", "Org", "", "null", ""]);
}

#[test]
fn the_name_falls_back_to_the_id_and_the_version_is_optional() {
    let text = r#"{"libraries":[{"uniqueId":"g:a","artifactVersion":"1.0"},
        {"uniqueId":"g:b","name":"B","artifactVersion":null}],"licenses":{}}"#;
    let parsed = data::parse(text);
    assert_eq!(names(&parsed), ["B", "g:a"]);
    assert_eq!(parsed.libraries[1].version.as_deref(), Some("1.0"));
    assert_eq!(parsed.libraries[0].version, None);
}

#[test]
fn a_malformed_resource_shows_no_libraries_as_the_parser_catch_all_does() {
    for bad in [
        "not json",
        r#"{"libraries":[]}"#,
        r#"{"libraries":[{"name":"x"}],"licenses":{}}"#,
        r#"{"libraries":[{"uniqueId":"x","developers":["text"]}],"licenses":{}}"#,
        r#"{"libraries":[{"uniqueId":"x","funding":[{"platform":"p"}]}],"licenses":{}}"#,
        r#"{"libraries":[],"licenses":{"MIT":{"content":"x"}}}"#,
    ] {
        assert_eq!(data::parse(bad), data::Libraries::default(), "{bad}");
    }
}

#[test]
fn licence_text_reads_as_html_from_html_gives_it() {
    let parsed = data::parse(&json(
        &[library("x", &["MIT", "Apache-2.0"], "")],
        &["Apache-2.0", "MIT"],
    ));
    let text = parsed.html(&parsed.libraries[0]);
    assert_eq!(text, "Apache-2.0 text<br /><br /><br /><br />MIT text");
    assert_eq!(html::from_html(&text), "Apache-2.0 text\n\n\n\nMIT text");
    // Each case was checked against Html.fromHtml (COMPACT) over TagSoup 1.2.
    for (input, expected) in [
        ("a  b\n c<br />d", "a b c\nd"),
        (
            "Copyright &copy; 2024 &lt;x&gt; &amp",
            "Copyright © 2024 <x> &",
        ),
        ("<p>one</p><p>two</p>", "one\ntwo\n"),
        ("x<script>a<b></script>y", "xa<b>y"),
        ("&#x1F600;&#65;&#0;&bogus;", "😀A&#0;&bogus;"),
        ("\r\n\u{85}\u{80}", "…€"),
        ("<ul><li>a</li><li>b</li></ul>", "a\nb\n"),
    ] {
        assert_eq!(html::from_html(input), expected, "{input:?}");
    }
}

fn screen(height: f32) -> (App, Text) {
    let mut a = App {
        config: [480.0, height, 1.0, 1.0],
        ..App::default()
    };
    a.navigate("settings/about");
    a.navigate(ROUTE);
    (a, roboto())
}
fn roboto() -> Text {
    Text::new([include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec()])
}
fn libraries(n: usize) -> String {
    let all: Vec<String> = (0..n)
        .map(|i| {
            library(
                &format!("Library {i:03}"),
                &["MIT"],
                r#","artifactVersion":"1.2.3","developers":[{"name":"Dev"}]"#,
            )
        })
        .collect();
    json(&all, &["MIT"])
}
fn rows(a: &App) -> Vec<i32> {
    a.controls
        .iter()
        .filter(|c| c.id >= ROW_ID)
        .map(|c| c.id)
        .collect()
}

#[test]
fn the_screen_asks_kotlin_for_the_resource_once() {
    let (mut a, mut text) = screen(640.0);
    assert_eq!(a.stack.last().map(String::as_str), Some(ROUTE));
    a.scene(&mut text);
    a.scene(&mut text);
    let loads: Vec<_> = a.effects.iter().filter(|e| e[0] == "load").collect();
    assert_eq!(loads, [&vec!["load".to_owned(), LIBRARIES.to_owned()]]);
    assert!(rows(&a).is_empty());
    a.effects.clear();
    a.set_document(LIBRARIES, &libraries(3));
    a.scene(&mut text);
    assert!(a.effects.is_empty());
    assert_eq!(rows(&a), [ROW_ID, ROW_ID + 1, ROW_ID + 2]);
}

#[test]
fn rows_say_their_name_version_author_and_licences() {
    let (mut a, mut text) = screen(640.0);
    a.set_document(LIBRARIES, &libraries(1));
    a.scene(&mut text);
    let row = a.controls.iter().find(|c| c.id == ROW_ID).unwrap();
    assert_eq!(row.label, "Library 000");
    assert_eq!(row.summary, "1.2.3, Dev, MIT name");
    assert_eq!(row.role, "card");
    assert!(row.enabled);
    assert_eq!(row.action, Action::Library(0));
}

#[test]
fn only_the_rows_on_screen_are_laid_out_and_drawn() {
    let (mut a, mut text) = screen(640.0);
    a.set_document(LIBRARIES, &libraries(500));
    let panel = a.scene(&mut text).panel.unwrap();
    let shown = rows(&a);
    assert!(!shown.is_empty() && shown.len() < 20, "{}", shown.len());
    assert_eq!(shown[0], ROW_ID);
    let names = panel
        .items
        .iter()
        .filter(|i| i.label.starts_with("Library "))
        .count();
    assert_eq!(names, shown.len());
    assert!(a.max_scroll > 100.0 * 480.0 / 10.0);
    // Halfway down, the rows drawn are the ones there.
    a.set_scroll(a.max_scroll / 2.0);
    a.scene(&mut text);
    let middle = rows(&a);
    assert!(
        middle[0] > ROW_ID + 200 && middle[0] < ROW_ID + 300,
        "{middle:?}"
    );
    assert!(middle.windows(2).all(|w| w[1] == w[0] + 1));
}

#[test]
fn keyboard_focus_walks_every_row_and_scrolls_it_into_view() {
    let (mut a, mut text) = screen(640.0);
    a.set_document(LIBRARIES, &libraries(50));
    a.scene(&mut text);
    a.focus_next(false);
    assert_eq!(a.focus, 1);
    for j in 0..50 {
        a.focus_next(false);
        assert_eq!(a.focus, ROW_ID + j);
        a.scene(&mut text);
        assert!(rows(&a).contains(&(ROW_ID + j)), "row {j} not on screen");
    }
    a.focus_next(false);
    assert_eq!(a.focus, 1);
}

#[test]
fn the_scrollbar_is_as_tall_as_the_first_frame_visible_rows() {
    let (mut a, mut text) = screen(640.0);
    a.set_document(LIBRARIES, &libraries(100));
    let bar = |a: &mut App, text: &mut Text| {
        let panel = a.scene(text).panel.unwrap();
        let bar = panel
            .items
            .iter()
            .find(|i| (i.bounds.width() - 4.0).abs() < 0.01 && i.bounds.left > 470.0)
            .map(|i| i.bounds)
            .expect("scrollbar");
        (bar, rows(a).len())
    };
    let (top, visible) = bar(&mut a, &mut text);
    a.set_scroll(a.max_scroll);
    let (end, _) = bar(&mut a, &mut text);
    assert!((top.height() - end.height()).abs() < 0.01);
    let viewport = top.height() * 100.0 / visible as f32;
    assert!(viewport > 500.0 && viewport < 640.0, "{viewport}");
    assert!(end.top > top.top);
}

#[test]
fn a_row_opens_its_licence_text_in_a_dialog_that_ok_closes() {
    let (mut a, mut text) = screen(640.0);
    a.set_document(LIBRARIES, &libraries(3));
    a.scene(&mut text);
    assert!(a.activate(ROW_ID + 1));
    a.scene(&mut text);
    let ids: Vec<i32> = a.controls.iter().map(|c| c.id).collect();
    assert_eq!(ids, [4, BODY_ID, 7]);
    let body = a.controls.iter().find(|c| c.id == BODY_ID).unwrap();
    assert_eq!(body.label, "MIT text");
    assert_eq!(body.role, "text");
    let ok = a.controls.iter().find(|c| c.id == 7).unwrap();
    assert_eq!((ok.label.as_str(), ok.role), ("action__ok", "button"));
    a.focus_next(false);
    assert_eq!(a.focus, BODY_ID);
    a.focus_next(false);
    assert_eq!(a.focus, 7);
    assert!(a.activate(7));
    assert!(a.modal.is_none());
    a.scene(&mut text);
    assert_eq!(rows(&a), [ROW_ID, ROW_ID + 1, ROW_ID + 2]);
}

#[test]
fn a_long_licence_scrolls_inside_the_dialog_and_draws_only_lines_on_screen() {
    let long = "line<br />".repeat(400);
    let text_json = format!(
        r#"{{"libraries":[{}],"licenses":{{"L":{{"name":"L","content":"{long}"}}}}}}"#,
        library("x", &["L"], "")
    );
    let (mut a, mut text) = screen(640.0);
    a.set_document(LIBRARIES, &text_json);
    a.scene(&mut text);
    a.activate(ROW_ID);
    let panel = a.scene(&mut text).panel.unwrap();
    let lines = panel.items.iter().filter(|i| i.label == "line").count();
    assert!(lines > 5 && lines < 60, "{lines}");
    assert!(a.max_scroll > 0.0);
    let ok = a.controls.iter().find(|c| c.id == 7).unwrap().bounds;
    assert!(ok.bottom <= 640.0);
}

#[test]
fn a_new_width_keeps_the_first_visible_row_at_the_top() {
    let (mut a, mut text) = screen(640.0);
    a.set_document(LIBRARIES, &libraries(200));
    a.scene(&mut text);
    a.set_scroll(a.max_scroll / 2.0);
    a.scene(&mut text);
    let first = rows(&a)[0];
    a.config = [900.0, 400.0, 1.0, 1.0];
    a.scene(&mut text);
    assert_eq!(rows(&a)[0], first);
}
