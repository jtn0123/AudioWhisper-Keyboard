//! The extension editor at its routes with Kotlin's reports, the strings it shows, and the taps
//! and typing a test needs.
pub use render::{
    settings_ui::{
        App, Control,
        catalogue::{Kind, ext_editor::Edit, extensions::Part, languages::Lang},
        extensions::editor::{
            ADD_ID, COMP_ID, COMPONENT_ADD, DEPENDENCIES, DEPENDENCY_ID, FILE, FILE_ADD,
            FILE_DELETE_ID, FILES, FIXED_ID, FOLDER_ID, FROM_ID, HEADER_ID, Load, MANAGE_ID, META,
            NO_DEPENDENCIES_ID, NONE_ID, SOURCE_ID, STATUS_ID,
        },
    },
    text::Text,
};
pub use wire::settings_ui::ext_editor::{effect, kind, outcome, row};
use wire::settings_ui::files::{document, done, picked, report};
pub use wire::settings_ui::files::{job, outcome as how, result};

pub const CREATE: &str = "ext/edit/null?create=ime.extension.theme";
pub const MINE: &str = "my.themes";
pub const EDIT: &str = "ext/edit/my.themes";
pub const WS: &str = "ws-1";
pub const NOW: &str = "1700000000000";
pub const FLAGS: &str = "TonalSpot/Default/SPEC_2021";

/// The extension `my.themes` as kotlinx encodes it: a day theme and a night theme that keeps
/// its stylesheet elsewhere.
pub const MANIFEST: &str = r#"{
  "$": "ime.extension.theme",
  "meta": {
    "id": "my.themes",
    "version": "1.0.0",
    "title": "My themes",
    "maintainers": [
      "Me"
    ],
    "license": "apache-2.0"
  },
  "themes": [
    {
      "id": "day",
      "label": "Day",
      "authors": [
        "Me"
      ],
      "isNight": false,
      "materialYouFlags": {
        "paletteStyle": "TonalSpot",
        "contrastLevel": "Default",
        "specVersion": "SPEC_2021"
      }
    },
    {
      "id": "night",
      "label": "Night",
      "authors": [
        "Me"
      ],
      "materialYouFlags": {
        "paletteStyle": "Vibrant",
        "contrastLevel": "High",
        "specVersion": "SPEC_2025"
      },
      "stylesheet": "stylesheets/dark.json"
    }
  ]
}"#;

/// A report row.
pub fn line(k: &str, [ext, id, label, detail, text, path]: [&str; 6], flag: bool) -> Vec<String> {
    let mut f = vec![String::new(); row::SIZE];
    f[row::KIND] = k.into();
    f[row::WORKSPACE] = WS.into();
    f[row::EXTENSION] = ext.into();
    f[row::ID] = id.into();
    f[row::LABEL] = label.into();
    f[row::DETAIL] = detail.into();
    f[row::TEXT] = text.into();
    f[row::PATH] = path.into();
    f[row::FLAG] = wire::flag::encode(flag).into();
    f
}
/// The workspace row: how opening extension `ext` ended, with `text` and whether the theme
/// editor changed it.
pub fn head(ext: &str, how: &str, text: &str, changed: bool) -> Vec<String> {
    line(kind::WORKSPACE, [ext, how, NOW, FLAGS, text, ""], changed)
}
/// The files, themes and installed extensions every ready report has.
pub fn rest() -> Vec<String> {
    [
        line(kind::FILE, ["", "fonts", "a.ttf", "", "", ""], false),
        line(kind::FILE, ["", "images", "bg.png", "", "", ""], false),
        line(
            kind::THEME,
            [
                "org.florisboard.themes",
                "floris_day",
                "Floris Day",
                "Neutral/Medium/SPEC_2021",
                "patrickgold\njtn",
                "stylesheets/floris_day.json",
            ],
            false,
        ),
        line(
            kind::THEME,
            [MINE, "night", "Night", FLAGS, "Me", "stylesheets/dark.json"],
            true,
        ),
        line(
            kind::INSTALLED,
            ["org.florisboard.themes", "", "", "", "", ""],
            false,
        ),
        line(kind::INSTALLED, [MINE, "", "", "", "", ""], false),
    ]
    .concat()
}
/// A ready report on extension `ext` holding `text`.
pub fn ready(ext: &str, text: &str, changed: bool) -> Vec<String> {
    [head(ext, outcome::READY, text, changed), rest()].concat()
}

pub fn text() -> Text {
    Text::new(vec![
        include_bytes!("../../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ])
}
/// The strings the editor shows, in English, and the icons it draws.
pub fn strings(a: &mut App) {
    for (k, v) in [
        ("ext__editor__title_create_theme", "Create theme extension"),
        ("ext__editor__title_edit_theme", "Edit theme extension"),
        ("ext__editor__metadata__title", "Manage meta data"),
        ("ext__editor__dependencies__title", "Manage dependencies"),
        ("ext__editor__files__title", "Manage archive files"),
        ("ext__editor__files__type_fonts", "Fonts"),
        ("ext__editor__files__type_images", "Images"),
        ("ext__editor__create_component__title_theme", "Create theme"),
        (
            "ext__editor__create_component__from_existing",
            "From existing",
        ),
        ("ext__editor__create_component__from_empty", "Empty"),
        (
            "ext__editor__create_component__from_empty_warning",
            "Empty warning.",
        ),
        ("ext__editor__metadata__title_invalid", "Invalid meta data"),
        (
            "ext__editor__metadata__message_invalid",
            "The meta data is invalid.",
        ),
        ("ext__meta__components_theme", "Bundled themes"),
        ("ext__meta__components_none_found", "No bundled components."),
        ("ext__meta__id", "ID"),
        ("ext__meta__version", "Version"),
        ("ext__meta__title", "Title"),
        ("ext__meta__description", "Description"),
        ("ext__meta__keywords", "Keywords"),
        ("ext__meta__homepage", "Homepage"),
        ("ext__meta__issue_tracker", "Issue tracker"),
        ("ext__meta__maintainers", "Maintainers"),
        ("ext__meta__license", "License"),
        ("ext__meta__label", "Label"),
        ("ext__meta__authors", "Authors"),
        (
            "ext__error__not_found_description",
            "No extension with ID \"{id}\" could be found.",
        ),
        ("action__add", "Add"),
        ("action__apply", "Apply"),
        ("action__cancel", "Cancel"),
        ("action__create", "Create"),
        ("action__delete", "Delete"),
        ("action__discard", "Discard"),
        ("action__edit", "Edit"),
        ("action__save", "Save"),
        ("action__import_file", "Import file"),
        ("action__discard_confirm_title", "Unsaved changes"),
        ("action__discard_confirm_message", "Discard the changes?"),
        ("general__file_name", "File name"),
        ("general__properties", "Properties"),
        (
            "settings__theme_editor__component_id_exists",
            "The ID is taken.",
        ),
        ("settings__ext_editor__opening", "Opening the extension…"),
        (
            "settings__ext_editor__unpack_failed",
            "It could not be unpacked.",
        ),
        ("settings__ext_editor__gone", "Its draft is gone."),
        (
            "settings__ext_editor__dependencies_note",
            "Dependencies note.",
        ),
        ("settings__ext_editor__no_dependencies", "No dependencies."),
        ("settings__ext_editor__file_name_invalid", "Bad file name."),
        ("settings__ext_editor__file_exists", "The file is there."),
        ("settings__ext_editor__wrong_type", "Wrong type."),
        ("settings__ext_editor__file_failed", "File failed."),
        ("settings__ext_editor__copy_failed", "Copy failed."),
        ("settings__ext_editor__save_failed", "Save failed."),
        (
            "settings__ext_editor__save_interrupted",
            "Save interrupted.",
        ),
        ("settings__ext_editor__id_taken", "The ID is installed."),
        (
            "settings__ext_editor__component_invalid",
            "Theme {name} is invalid.",
        ),
        ("settings__udm__no_picker", "No picker."),
        ("settings__files__interrupted", "Interrupted."),
    ] {
        a.strings.insert(k.into(), v.into());
    }
    for icon in [
        "add",
        "edit",
        "delete",
        "code",
        "library_books",
        "file_blank",
        "text_fields",
        "photo",
        "info",
        "arrow_back",
    ] {
        a.strings
            .insert(format!("@icon:{icon}"), "M0 0h24v24H0z".into());
    }
}
/// A phone-sized app.
pub fn fresh() -> (App, Text) {
    let mut a = App {
        config: [1080.0, 6000.0, 2.625, 1.0],
        ..App::default()
    };
    strings(&mut a);
    (a, text())
}
/// The editor at `route` before Kotlin answers, its effects cleared after checking it asked.
pub fn asked(route: &str, ext: &str) -> (App, Text) {
    let (mut a, mut text) = fresh();
    a.navigate(route);
    assert_eq!(
        last(&a, "extEditor"),
        Some(vec![
            "extEditor".into(),
            ext.into(),
            String::new(),
            String::new()
        ])
    );
    a.scene(&mut text);
    a.effects.clear();
    (a, text)
}
/// The editor on `my.themes`, open and painted, its effects cleared.
pub fn editing() -> (App, Text) {
    let (mut a, mut text) = asked(EDIT, MINE);
    a.set_ext_editor(&ready(MINE, MANIFEST, false)).unwrap();
    a.scene(&mut text);
    a.effects.clear();
    (a, text)
}
/// The editor creating a theme extension, open and painted, its effects cleared.
pub fn creating() -> (App, Text) {
    let (mut a, mut text) = asked(CREATE, "");
    a.set_ext_editor(&ready("", "", false)).unwrap();
    a.scene(&mut text);
    a.effects.clear();
    (a, text)
}
pub fn control(a: &App, id: i32) -> Control {
    a.controls
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .unwrap_or_else(|| panic!("no control {id}"))
}
pub fn has(a: &App, id: i32) -> bool {
    a.controls.iter().any(|c| c.id == id)
}
/// Paints, taps `id` as it now stands, and paints again.
pub fn tap(a: &mut App, text: &mut Text, id: i32) -> bool {
    a.scene(text);
    let done = a.activate(id);
    a.scene(text);
    done
}
/// The last packet of effect `kind`.
pub fn last(a: &App, kind: &str) -> Option<Vec<String>> {
    a.effects.iter().rev().find(|e| e[0] == kind).cloned()
}
/// The last toast's text.
pub fn toast(a: &App) -> Option<String> {
    last(a, "toast").map(|t| t[1].clone())
}
pub fn row_id(a: &App, pick: impl Fn(&Kind) -> bool) -> i32 {
    1000 + a.rows().iter().position(|r| pick(&r.kind)).unwrap() as i32
}
pub fn edit_id(a: &App, e: Edit) -> i32 {
    row_id(a, |k| matches!(k, Kind::Ext(Part::Editor(q)) if *q == e))
}
pub fn lang_id(a: &App, l: Lang) -> i32 {
    row_id(a, |k| matches!(k, Kind::Lang(q) if *q == l))
}
/// Taps the bottom bar's confirming button: Save, Apply, Create or Add.
pub fn confirm(a: &mut App, text: &mut Text) -> bool {
    let id = lang_id(a, Lang::Save);
    tap(a, text, id)
}
/// Taps the bottom bar's Cancel.
pub fn cancel(a: &mut App, text: &mut Text) -> bool {
    let id = lang_id(a, Lang::Cancel);
    tap(a, text, id)
}
/// The route on top.
pub fn top(a: &App) -> &str {
    a.stack.last().map_or("", String::as_str)
}
/// Focuses line field `e`, types `value` over what it held, and paints.
pub fn type_into(a: &mut App, text: &mut Text, e: Edit, value: &str) {
    let id = edit_id(a, e);
    assert!(tap(a, text, id), "{e:?}");
    a.editor.text = value.encode_utf16().collect();
    a.editor.anchor = a.editor.text.len();
    a.editor.focus = a.editor.text.len();
    a.sync_field();
    a.scene(text);
}
/// The `work` effect's fields by name: job, then extension, document, text, name and target.
pub fn work(a: &App) -> Option<[String; 6]> {
    use wire::settings_ui::files::work as w;
    let p = last(a, "work")?;
    Some(
        [
            w::JOB,
            w::EXTENSION,
            w::DOCUMENT,
            w::TEXT,
            w::NAME,
            w::TARGET,
        ]
        .map(|i| p[i].clone()),
    )
}
/// A `DONE` report for `job`: its result, name and text.
pub fn done_report(job: &str, how: &str, name: &str, from: &str) -> Vec<String> {
    let mut p = vec![String::new(); done::SIZE];
    p[done::REPORT] = report::DONE.into();
    p[done::JOB] = job.into();
    p[done::RESULT] = how.into();
    p[done::NAME] = name.into();
    p[done::TEXT] = from.into();
    p
}
/// Kotlin's queue is idle: work Rust waited for was lost with the activity.
pub fn idle() -> Vec<String> {
    vec![report::IDLE.into()]
}
/// The manifest with `from` replaced by `to`.
pub fn manifest_with(from: &str, to: &str) -> String {
    assert!(MANIFEST.contains(from), "{from}");
    MANIFEST.replacen(from, to, 1)
}
pub fn picked_report(how: &str, docs: &[(&str, &str)]) -> Vec<String> {
    let mut p = vec![String::new(); picked::SIZE];
    p[picked::REPORT] = report::PICKED.into();
    p[picked::OUTCOME] = how.into();
    for (handle, name) in docs {
        let mut d = vec![String::new(); document::SIZE];
        d[document::HANDLE] = (*handle).into();
        d[document::NAME] = (*name).into();
        p.extend(d);
    }
    p
}
/// The control ids focus visits, from Back round to Back. A focus that never comes back to Back,
/// stuck on one control or circling some of them, fails instead of walking for ever.
pub fn focus_order(a: &mut App) -> Vec<i32> {
    a.focus = -1;
    a.focus_next(false);
    let first = a.focus;
    let mut out = vec![first];
    for _ in 0..FOCUS_STEPS {
        a.focus_next(false);
        if a.focus == first {
            return out;
        }
        out.push(a.focus);
    }
    panic!("focus never came back to {first}: {:?}…", &out[..32]);
}
/// More focus steps than any editor screen has controls.
const FOCUS_STEPS: usize = 500;
/// The theme ids in the draft.
pub fn theme_ids(a: &App) -> Vec<String> {
    let themes = &a.extensions.editor.draft.themes;
    themes.iter().map(|t| t.id.clone()).collect()
}
