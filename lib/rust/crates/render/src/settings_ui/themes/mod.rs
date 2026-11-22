//! The theme manager, drawn by Rust at ThemeManagerScreen's routes: every installed theme
//! extension in an outlined box listing its themes by label, the chosen one's radio button on; a
//! tap chooses the day or the night theme (`manager.rs`). While it is on screen the keyboard
//! shows the chosen theme, as ThemeManagerScreen's DisposableEffect had it. Kotlin reports the
//! extension index (`wire::settings_ui::themes`) and sets the keyboard's preview; Rust groups,
//! sorts and draws the themes and writes the preference. Creating, editing and importing a theme
//! and an extension's own page are Compose's extension screens, which the manager hands off to.
pub mod manager;
use super::App;
use wire::settings_ui::themes::{effect, kind, row};

pub const DAY: &str = "settings/theme/manage/SELECT_DAY";
pub const NIGHT: &str = "settings/theme/manage/SELECT_NIGHT";
/// The extension list's Create extension for themes: `Routes.Ext.Edit("null", SERIAL_TYPE)`.
pub const CREATE: &str = "ext/edit/null?create=ime.extension.theme";
/// The theme extension list's Import box.
pub const IMPORT: &str = "ext/import/EXT_THEME";
/// The strings the manager shows beyond its titles.
pub const KEYS: [&str; 6] = [
    "ext__editor__title_create_theme",
    "action__import",
    "ext__list__view_details",
    "action__edit",
    "pref__theme__day",
    "pref__theme__night",
];

/// A theme, as ThemeExtensionComponent names it.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub id: String,
    pub label: String,
    pub night: bool,
}

/// A theme extension and its themes, sorted by label.
#[derive(Clone, Debug, PartialEq)]
pub struct Extension {
    pub id: String,
    pub title: String,
    /// Whether the extension manager can delete it: it was imported or made here.
    pub deletable: bool,
    pub themes: Vec<Theme>,
}

/// The theme manager's own state.
#[derive(Debug, Default)]
pub struct Themes {
    /// The last report, grouped and sorted; None until one comes.
    pub extensions: Option<Vec<Extension>>,
    /// A report was asked for and has not come.
    pub requested: bool,
    /// The theme the keyboard was last told to show; None while it shows its own.
    pub preview: Option<String>,
}

/// A tap on the theme manager's own controls.
#[derive(Clone, Debug, PartialEq)]
pub enum Tap {
    /// Theme `t` of extension `e` chosen.
    Pick(usize, usize),
    /// An extension's page, where it is exported or deleted: its title, id or View details.
    View(String),
    /// An extension made or imported here, in the extension editor.
    Edit(String),
    Create,
    Import,
}

/// A `THEMES` report as ThemeManagerScreen grouped it: by extension id in the index's order, a
/// repeated id keeping its first place and taking the later extension, as `buildMap` does, and
/// each extension's themes sorted by label as Kotlin's `sortedBy` sorts Strings, stably.
pub fn parse(p: &[String]) -> Result<Vec<Extension>, &'static str> {
    if !p.len().is_multiple_of(row::SIZE) {
        return Err("settings themes");
    }
    let mut out: Vec<Extension> = vec![];
    let mut at = None;
    for f in p.chunks(row::SIZE) {
        let flag = |i: usize| f[i] == wire::flag::TRUE;
        match f[row::KIND].as_str() {
            kind::EXTENSION => {
                let ext = Extension {
                    id: f[row::EXTENSION].clone(),
                    title: f[row::LABEL].clone(),
                    deletable: flag(row::DELETABLE),
                    themes: vec![],
                };
                at = Some(match out.iter().position(|e| e.id == ext.id) {
                    Some(i) => {
                        out[i] = ext;
                        i
                    }
                    None => {
                        out.push(ext);
                        out.len() - 1
                    }
                });
            }
            kind::THEME => {
                let e = at.ok_or("settings themes first")?;
                if out[e].id != f[row::EXTENSION] {
                    return Err("settings themes extension");
                }
                out[e].themes.push(Theme {
                    id: f[row::COMPONENT].clone(),
                    label: f[row::LABEL].clone(),
                    night: flag(row::NIGHT),
                });
            }
            _ => return Err("settings themes kind"),
        }
    }
    for e in &mut out {
        e.themes
            .sort_by(|a, b| a.label.encode_utf16().cmp(b.label.encode_utf16()));
    }
    Ok(out)
}

impl App {
    /// A `THEMES` report: the installed theme extensions as they are now.
    pub fn set_themes(&mut self, p: &[String]) -> Result<(), &'static str> {
        self.themes.extensions = Some(parse(p)?);
        self.themes.requested = false;
        Ok(())
    }
    /// The preference the manager on screen chooses, the day or the night theme's.
    pub fn theme_key(&self) -> Option<&'static str> {
        match self.stack.last()?.as_str() {
            DAY => Some("theme__day_theme_id"),
            NIGHT => Some("theme__night_theme_id"),
            _ => None,
        }
    }
    /// The installed theme extensions, as the last report gave them.
    pub fn theme_extensions(&self) -> &[Extension] {
        self.themes.extensions.as_deref().unwrap_or(&[])
    }
    /// Whether theme `t` of extension `e` is the one the preference names.
    pub fn theme_chosen(&self, e: usize, t: usize) -> bool {
        let (Some(key), Some(ext)) = (self.theme_key(), self.theme_extensions().get(e)) else {
            return false;
        };
        ext.themes
            .get(t)
            .is_some_and(|theme| self.value(key) == format!("{}:{}", ext.id, theme.id))
    }
    /// Tells the keyboard which theme to show: the chosen one while a manager is on top, its own
    /// once none is. Only a change is sent.
    pub(super) fn sync_theme_preview(&mut self) {
        let wanted = self.theme_key().map(|key| self.value(key));
        if wanted != self.themes.preview {
            let name = wanted.clone().unwrap_or_default();
            self.effects.push(vec![effect::PREVIEW.into(), name]);
            self.themes.preview = wanted;
        }
    }
    /// Asks Kotlin for the installed themes unless that report is on its way.
    fn request_themes(&mut self) {
        if !self.themes.requested {
            self.themes.requested = true;
            self.effects.push(vec![effect::LOAD.into()]);
        }
    }
    /// A native screen opened at `route`: a manager reads the themes as they are now.
    pub(super) fn themes_opened(&mut self, route: &str) {
        if matches!(route, DAY | NIGHT) {
            self.request_themes();
        }
        self.sync_theme_preview();
    }
    /// Back went to the screen now on top.
    pub(super) fn themes_left(&mut self) {
        self.sync_theme_preview();
    }
    /// After a saved state came back: the report and the keyboard's preview again.
    pub fn themes_restored(&mut self) {
        self.themes.requested = false;
        self.themes.preview = None;
        if self.theme_key().is_some() {
            self.request_themes();
        }
        self.sync_theme_preview();
    }
    /// A tap on the theme manager's own control.
    pub(super) fn themes_tap(&mut self, tap: Tap) {
        match tap {
            Tap::Pick(e, t) => self.pick_theme(e, t),
            Tap::View(id) => self.navigate(&format!("ext/view/{id}")),
            Tap::Edit(id) => self.navigate(&format!("ext/edit/{id}")),
            Tap::Create => self.navigate(CREATE),
            Tap::Import => self.navigate(IMPORT),
        }
    }
    /// Writes theme `t` of extension `e` as the day or night theme, as `ExtensionComponentName`
    /// stores it, and shows it on the keyboard.
    fn pick_theme(&mut self, e: usize, t: usize) {
        let (Some(key), Some(ext)) = (self.theme_key(), self.theme_extensions().get(e)) else {
            return;
        };
        let Some(theme) = ext.themes.get(t) else {
            return;
        };
        let value = format!("{}:{}", ext.id, theme.id);
        self.write(key, value);
        self.sync_theme_preview();
    }
}
