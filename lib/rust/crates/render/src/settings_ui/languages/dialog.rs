//! The language screens' dialogs, drawn by the settings dialog (`settings_ui/dialog.rs`): a
//! field's choices and every preset, as a list that applies a tap and closes, where Compose had a
//! dropdown menu and a list dialog; the editor's error; and the language pack delete question.
use crate::settings_ui::{
    App,
    catalogue::{Kind, languages::Lang},
};
use wire::settings_ui::languages::effect;

impl App {
    /// Opens the dialog row `l` of the screen on top, with `selected`: a message's string key, or
    /// the language pack to delete.
    pub(in crate::settings_ui) fn open_lang_dialog(&mut self, l: Lang, selected: &str) {
        let at = self
            .rows()
            .iter()
            .position(|r| matches!(r.kind, Kind::Lang(k) if k == l));
        if let Some(i) = at {
            self.open_modal(i, selected.into());
        }
    }
    /// Whether the dialog lists every preset rather than a field's choices.
    pub fn presets_dialog(&self) -> bool {
        self.modal
            .as_ref()
            .is_some_and(|m| matches!(self.rows()[m.row].kind, Kind::Lang(Lang::AllPresets)))
    }
    /// What row `i`'s dialog lists, when it is a language dialog: component names, or preset
    /// indices.
    pub(in crate::settings_ui) fn lang_choices(&self, i: usize) -> Option<Vec<String>> {
        match self.rows().get(i)?.kind {
            Kind::Pick(f) => Some(self.pick_options(f).into_iter().map(|o| o.0).collect()),
            Kind::Lang(Lang::AllPresets) => {
                let n = self
                    .languages
                    .resources
                    .as_ref()
                    .map_or(0, |r| r.presets.len());
                Some((0..n).map(|k| k.to_string()).collect())
            }
            Kind::ThemeEdit(_) => self.theme_choices(i),
            _ => None,
        }
    }
    /// A choice's label: the component's, or the preset's language.
    pub(in crate::settings_ui) fn lang_choice_label(
        &self,
        i: usize,
        choice: &str,
    ) -> Option<String> {
        match self.rows().get(i)?.kind {
            Kind::Pick(f) => Some(
                self.pick_options(f)
                    .into_iter()
                    .find(|o| o.0 == choice)
                    .map_or_else(|| choice.to_owned(), |o| o.1),
            ),
            Kind::Lang(Lang::AllPresets) => Some(self.preset_text(choice.parse().ok()?).0),
            Kind::ThemeEdit(_) => self.theme_choice_label(i, choice),
            _ => None,
        }
    }
    /// A choice's second line: a preset's characters layout; none for a field.
    pub(in crate::settings_ui) fn lang_choice_description(
        &self,
        i: usize,
        choice: &str,
    ) -> Option<String> {
        match self.rows().get(i)?.kind {
            Kind::Pick(_) | Kind::ThemeEdit(_) => Some(String::new()),
            Kind::Lang(Lang::AllPresets) => Some(self.preset_text(choice.parse().ok()?).1),
            _ => None,
        }
    }
    /// A tap on a language dialog's choice: applied, and the dialog closed. Any other dialog's
    /// choice comes back to be selected.
    pub(in crate::settings_ui) fn lang_choose(&mut self, choice: String) -> Option<String> {
        let row = self.modal.as_ref()?.row;
        match self.rows()[row].kind {
            Kind::Pick(f) => {
                if let Some(d) = &mut self.draft {
                    d.set(f, &choice);
                }
            }
            Kind::Lang(Lang::AllPresets) => {
                if let Ok(k) = choice.parse() {
                    self.apply_preset(k);
                }
            }
            Kind::ThemeEdit(_) => self.theme_choose(row, &choice),
            _ => return Some(choice),
        }
        self.modal = None;
        self.focus = -1;
        None
    }
    /// The confirm button of a language dialog: OK closes the error, Delete deletes the pack;
    /// the theme editor's Discard leaves it and its Delete deletes the rule being edited.
    pub(in crate::settings_ui) fn lang_confirmed(&mut self, l: Lang, selected: String) {
        match l {
            Lang::DeletePack => self
                .effects
                .push(vec![effect::DELETE_PACK.into(), selected]),
            Lang::Discard => self.discard_theme(),
            Lang::DeleteRule => self.delete_form_rule(),
            _ => {}
        }
    }
    /// The message a language dialog shows: the error, or the question naming the pack.
    pub(in crate::settings_ui) fn lang_message(&self) -> Option<String> {
        let m = self.modal.as_ref()?;
        match self.rows()[m.row].kind {
            Kind::Lang(Lang::Message) => Some(self.label(&m.selected)),
            Kind::Lang(Lang::Discard) => Some(self.label(self.rows()[m.row].summary)),
            Kind::Lang(Lang::DeleteRule) => {
                let original = self.theme_editor.form.as_ref()?.original.as_ref()?;
                let name = self.rule_lines(original).0;
                Some(
                    self.label(self.rows()[m.row].summary)
                        .replace("{name}", &name),
                )
            }
            Kind::Lang(Lang::DeletePack) => {
                let packs = self
                    .languages
                    .resources
                    .as_ref()
                    .map_or(&[][..], |r| &r.packs[..]);
                let title = packs
                    .iter()
                    .find(|p| p.id == m.selected)
                    .map_or(m.selected.as_str(), |p| p.title.as_str());
                Some(
                    self.label(self.rows()[m.row].summary)
                        .replace("{name}", title),
                )
            }
            _ => None,
        }
    }
    /// The dialog's buttons, as neutral, dismiss and confirm string keys.
    pub(in crate::settings_ui) fn lang_buttons(&self) -> Option<[&'static str; 3]> {
        let m = self.modal.as_ref()?;
        match self.rows()[m.row].kind {
            Kind::Pick(_) | Kind::Lang(Lang::AllPresets) | Kind::ThemeEdit(_) => {
                Some(["", "action__cancel", ""])
            }
            Kind::Lang(Lang::Message) => Some(["", "", "action__ok"]),
            Kind::Lang(Lang::DeletePack | Lang::DeleteRule) => {
                Some(["", "action__cancel", "action__delete"])
            }
            Kind::Lang(Lang::Discard) => Some(["", "action__cancel", "action__discard"]),
            _ => None,
        }
    }
}
