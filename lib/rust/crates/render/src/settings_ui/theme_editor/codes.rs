//! The code editor, EditCodeValueDialog as a screen over the rule editor: a code chip opens it on
//! that code and the codes' Add chip on none. It types a code, checked against the ranges a key
//! code has, or records one: the next key-up on this app's own keyboard sets it, which Kotlin
//! takes from the running input method (`wire::settings_ui::theme_codes`). Recording needs that
//! keyboard enabled and selected; when it is not, the editor says so under the field and opens
//! the system's input method settings or its picker. The field takes several codes, comma
//! separated, so a run of codes need not open the editor once each; a recorded key replaces the
//! one being typed. A recorded or typed code is typed content: it goes nowhere but the rule.
use super::{
    Tap,
    codes_preview::{Described, key_data, shown_code},
    paint::DELETE_ID,
};
use crate::settings_ui::{
    Action, App,
    catalogue::{
        Kind,
        theme_editor::{CodeRow, Part},
    },
};
use keys::codes::{CHARACTERS_MAX, CHARACTERS_MIN, INTERNAL_MAX, INTERNAL_MIN};
use wire::settings_ui::theme_codes::{effect, kind, report};

pub const CODE_ADD: &str = "settings/theme/editor/code/add";
pub const CODE_EDIT: &str = "settings/theme/editor/code";
/// Why Add or Apply refused, and why recording did not start.
pub const INVALID_CODE: &str = "settings__theme_editor__code_invalid";
pub const EXISTS: &str = "settings__theme_editor__code_already_exists";
pub const NEEDS_KEYBOARD: &str =
    "settings__theme_editor__code_recording_requires_default_ime_floris";
/// The help's links, named as Compose named them, in English, and the strings with their URLs.
pub const LINKS: [(&str, &str); 2] = [
    (
        "Characters (unicode-table.com)",
        "florisboard__character_key_codes_url",
    ),
    (
        "Internal (github.com)",
        "florisboard__internal_key_codes_url",
    ),
];
/// The strings the code editor shows beyond its rows'.
pub const KEYS: [&str; 11] = [
    EXISTS,
    NEEDS_KEYBOARD,
    "settings__theme_editor__code_placeholder",
    "settings__theme_editor__code_recording_placeholder",
    "settings__theme_editor__code_recording_help_text",
    "settings__theme_editor__code_help_text",
    "settings__theme_editor__code_record",
    "settings__theme_editor__add_code",
    "settings__theme_editor__edit_code",
    LINKS[0].1,
    LINKS[1].1,
];

/// A tap on the code editor's or the codes chips' controls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodeTap {
    /// The rule editor's code chip `j`, or its Add chip.
    Open(Option<usize>),
    /// *Record a key*, which stops a recording in progress.
    Record,
    /// The top bar's Delete, editing a code.
    Delete,
    /// The help's link `k`.
    Link(usize),
}

/// The code editor's state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CodeForm {
    /// The code being changed; None while adding.
    pub original: Option<String>,
    /// The field: one code, or several separated by commas.
    pub text: String,
    /// Waiting for a key from this app's keyboard. A saved state does not keep it: the activity
    /// that comes back listens to nothing, and a key pressed later is not the one asked for.
    pub recording: bool,
    /// Why Add or Apply refused, or that recording needs this app's keyboard: a string key, or
    /// empty.
    pub error: &'static str,
    /// What Kotlin said of the code the preview shows, or that it was asked.
    pub said: Option<Described>,
}

/// The codes `field` gives, each as a rule writes it, or the string key of why it gives none:
/// one out of range, or one the rule `has` already, other than the `original` being edited.
pub fn checked_codes(
    field: &str,
    original: Option<&str>,
    has: &[String],
) -> Result<Vec<String>, &'static str> {
    let entries: Vec<&str> = field
        .split(',')
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .collect();
    if entries.is_empty() {
        return Err(INVALID_CODE);
    }
    let mut codes = vec![];
    for entry in entries {
        let code = entry.parse::<i32>().ok().filter(|c| {
            (CHARACTERS_MIN..=CHARACTERS_MAX).contains(c)
                || (INTERNAL_MIN..=INTERNAL_MAX).contains(c)
        });
        codes.push(code.ok_or(INVALID_CODE)?.to_string());
    }
    let taken = |c: &String| has.contains(c) && original != Some(c.as_str());
    if codes.iter().any(taken) {
        return Err(EXISTS);
    }
    Ok(codes)
}

/// `field` with its last entry, the one being typed, replaced by `code`.
pub fn with_recorded(field: &str, code: i32) -> String {
    match field.rfind(',') {
        Some(at) => format!("{}, {code}", field[..at].trim_end()),
        None => code.to_string(),
    }
}

impl App {
    /// A tap on the code editor's or the codes chips' control.
    pub(super) fn code_tap(&mut self, tap: CodeTap) {
        match tap {
            CodeTap::Open(j) => self.open_code(j),
            CodeTap::Record => self.record_key(),
            CodeTap::Delete => self.delete_code(),
            CodeTap::Link(k) => {
                let url = self.label(LINKS[k.min(1)].1);
                self.effects.push(vec!["url".into(), url]);
            }
        }
    }
    /// The editor over the rule editor, showing its code `j` or adding one.
    fn open_code(&mut self, j: Option<usize>) {
        let Some(rule) = self.theme_editor.form.as_ref() else {
            return;
        };
        let original = match j {
            Some(j) => match rule.values(0).get(j) {
                Some(code) => Some(code.clone()),
                None => return,
            },
            None => None,
        };
        let route = if original.is_some() {
            CODE_EDIT
        } else {
            CODE_ADD
        };
        let text = original.clone().unwrap_or_default();
        self.theme_editor.code = Some(CodeForm {
            original,
            text,
            ..CodeForm::default()
        });
        self.navigate(route);
        self.describe_code();
    }
    /// The row the code field is on, on this screen.
    fn code_field_row(&self) -> Option<usize> {
        let field = |k: &Kind| matches!(k, Kind::ThemeEdit(Part::Code(CodeRow::Field)));
        self.rows().iter().position(|r| field(&r.kind))
    }
    /// *Record a key*: listening starts, with the field focused so the keyboard shows; tapped
    /// while listening, it stops. Without this app's keyboard enabled and selected, the editor
    /// says so and opens the system's input method settings, or its picker.
    fn record_key(&mut self) {
        let ready = self.ime_enabled && self.ime_selected;
        let Some(f) = self.theme_editor.code.as_mut() else {
            return;
        };
        if f.recording {
            return self.stop_recording();
        }
        if !ready {
            f.error = NEEDS_KEYBOARD;
            let open = if self.ime_enabled { "picker" } else { "enable" };
            return self.effects.push(vec![open.into()]);
        }
        (f.recording, f.error) = (true, "");
        self.effects.push(vec![effect::RECORD.into()]);
        if let Some(i) = self.code_field_row() {
            self.own_field(Some(i));
            self.editor.focused = true;
            self.effects.push(vec!["editor".into()]);
        }
    }
    /// Listening stops, if it was on.
    fn stop_recording(&mut self) {
        if let Some(f) = self.theme_editor.code.as_mut().filter(|f| f.recording) {
            f.recording = false;
            self.effects.push(vec![effect::STOP.into()]);
        }
    }
    /// A `THEME_CODE` report: the key recorded, recording stopped, or a code described.
    pub fn theme_code(&mut self, p: &[String]) -> Result<(), &'static str> {
        const ERROR: &str = "settings theme code";
        if p.len() != report::SIZE {
            return Err(ERROR);
        }
        let code = || p[report::CODE].parse::<i32>().map_err(|_| ERROR);
        match p[report::KIND].as_str() {
            kind::KEY => self.code_recorded(code()?),
            kind::STOPPED => {
                if let Some(f) = self.theme_editor.code.as_mut() {
                    f.recording = false;
                }
            }
            kind::DESCRIBED => {
                let said = Described {
                    code: code()?,
                    name: p[report::NAME].clone(),
                    non_spacing: p[report::NON_SPACING] == wire::flag::TRUE,
                    icon: icons::key_path(p[report::ICON].parse().unwrap_or(0)).to_owned(),
                };
                let f = self.theme_editor.code.as_mut();
                if let Some(f) = f.filter(|f| f.said.as_ref().is_some_and(|s| s.code == said.code))
                {
                    f.said = Some(said);
                }
            }
            _ => return Err(ERROR),
        }
        Ok(())
    }
    /// The key recorded: its code replaces the one being typed. A key while not recording, as
    /// one that was on its way when recording stopped, does nothing.
    fn code_recorded(&mut self, code: i32) {
        let Some(f) = self.theme_editor.code.as_mut().filter(|f| f.recording) else {
            return;
        };
        f.text = with_recorded(&f.text, code);
        (f.recording, f.error) = (false, "");
        self.refresh_owned_field();
        self.describe_code();
    }
    /// Asks Kotlin about the code the preview shows, unless it was asked already. The
    /// unspecified key has no name or icon to ask for.
    pub(super) fn describe_code(&mut self) {
        let Some(f) = self.theme_editor.code.as_mut() else {
            return;
        };
        let code = key_data(shown_code(&f.text)).code;
        if code == 0 || f.said.as_ref().is_some_and(|s| s.code == code) {
            return;
        }
        f.said = Some(Described {
            code,
            ..Described::default()
        });
        let ask = vec![effect::DESCRIBE.into(), code.to_string()];
        self.effects.push(ask);
    }
    /// The code field changed: a message goes, and so does recording, since the code is
    /// being typed.
    pub(super) fn code_typed(&mut self, value: String) {
        let Some(f) = self.theme_editor.code.as_mut() else {
            return;
        };
        if f.text == value {
            return;
        }
        (f.text, f.error) = (value, "");
        self.stop_recording();
        self.describe_code();
    }
    /// Add or Apply: the codes replace the one being edited, or join the rule's, unless one is
    /// out of range or the rule has it; an unchanged code just closes, as the dialog's did.
    pub(super) fn save_code(&mut self) {
        let e = &mut self.theme_editor;
        let (Some(f), Some(rule)) = (e.code.as_mut(), e.form.as_mut()) else {
            return;
        };
        match checked_codes(&f.text, f.original.as_deref(), &rule.values(0)) {
            Err(key) => f.error = key,
            Ok(codes) => {
                if let Some(old) = &f.original {
                    rule.set_value(0, old, false);
                }
                for code in codes {
                    rule.set_value(0, &code, true);
                }
                self.close_editor();
            }
        }
    }
    /// The top bar's Delete: the code goes from the rule, and the editor closes.
    fn delete_code(&mut self) {
        let e = &mut self.theme_editor;
        let original = e.code.as_ref().and_then(|f| f.original.clone());
        if let (Some(rule), Some(old)) = (e.form.as_mut(), original) {
            rule.set_value(0, &old, false);
        }
        self.close_editor();
    }
    /// Back left the code editor, onto the rule editor: listening stops. False when it was not
    /// open.
    pub(super) fn code_editor_left(&mut self) -> bool {
        self.stop_recording();
        self.theme_editor.code.take().is_some()
    }
    /// The field's message: why Add or Apply refused, that a key is awaited, or that recording
    /// needs this app's keyboard, until it is enabled and selected.
    pub fn code_message(&self) -> String {
        let Some(f) = self.theme_editor.code.as_ref() else {
            return String::new();
        };
        let ready = self.ime_enabled && self.ime_selected;
        match f.error {
            _ if f.recording => self.label("settings__theme_editor__code_recording_placeholder"),
            NEEDS_KEYBOARD if ready => String::new(),
            "" => String::new(),
            key => self.code_ranges(key),
        }
    }
    /// String `key` with the code ranges `{c_min}`, `{c_max}`, `{i_min}` and `{i_max}` filled.
    pub fn code_ranges(&self, key: &str) -> String {
        self.label(key)
            .replace("{c_min}", &CHARACTERS_MIN.to_string())
            .replace("{c_max}", &CHARACTERS_MAX.to_string())
            .replace("{i_min}", &INTERNAL_MIN.to_string())
            .replace("{i_max}", &INTERNAL_MAX.to_string())
    }
    /// The top bar's Delete, editing a code: the code, for TalkBack.
    pub fn code_header(&self) -> Option<(i32, &'static str, &'static str, String, Action)> {
        let original = self.theme_editor.code.as_ref()?.original.clone()?;
        let delete = Action::ThemeEdit(Tap::Code(CodeTap::Delete));
        Some((DELETE_ID, "delete", "action__delete", original, delete))
    }
}
