//! The user dictionary screens, drawn by Rust at UserDictionaryScreen's route: the languages words
//! are stored under (`form.rs`), a language's words as the editable list (`list.rs`), the word
//! editor (`editor.rs`) and import and export (`transfer.rs`), for the Floris and the system
//! dictionary alike. Kotlin reports a dictionary's words (`wire::settings_ui::words`) and carries
//! out each change; Rust decides what is shown, checks a word, reads an import and writes an
//! export's text.
pub mod editor;
pub mod form;
pub mod list;
pub mod saved;
pub mod transfer;
use super::App;
use wire::settings_ui::words::{dictionary, effect, kind, outcome, row};

const ROOT: &str = "settings/dictionary/user-dictionary/";
pub const FLORIS: &str = "settings/dictionary/user-dictionary/FLORIS";
pub const SYSTEM: &str = "settings/dictionary/user-dictionary/SYSTEM";
/// A language's words: `/` and the language's [`group`] follow, or nothing for all languages.
pub const FLORIS_WORDS: &str = "settings/dictionary/user-dictionary/FLORIS/words";
pub const SYSTEM_WORDS: &str = "settings/dictionary/user-dictionary/SYSTEM/words";
pub const FLORIS_ADD: &str = "settings/dictionary/user-dictionary/FLORIS/word/add";
pub const SYSTEM_ADD: &str = "settings/dictionary/user-dictionary/SYSTEM/word/add";
/// The editor of a stored word: its id follows as one more path segment.
pub const FLORIS_EDIT: &str = "settings/dictionary/user-dictionary/FLORIS/word/edit";
pub const SYSTEM_EDIT: &str = "settings/dictionary/user-dictionary/SYSTEM/word/edit";
/// The strings these screens show beyond their rows'.
pub const KEYS: [&str; 15] = [
    "settings__udm__all_languages",
    "settings__udm__word_summary_freq",
    "settings__udm__word_summary_freq_shortcut",
    "settings__udm__dialog__word_error_empty",
    "settings__udm__dialog__word_error_invalid",
    "settings__udm__dialog__freq_error_empty",
    "settings__udm__dialog__freq_error_invalid",
    "settings__udm__dialog__shortcut_error_invalid",
    "settings__udm__word_error_exists",
    "settings__udm__dictionary_import_success",
    "settings__udm__dictionary_export_success",
    "settings__udm__import_line_error",
    "settings__udm__file_error",
    "settings__udm__no_picker",
    "settings__udm__change_failed",
];

/// Where a route is on the user dictionary screens.
#[derive(Clone, Debug, PartialEq)]
pub enum Page {
    Languages,
    /// A language's words, by its [`group`]: None for all languages.
    Words(Option<String>),
    Add,
    Edit(i64),
}

/// A stored word, as `UserDictionaryEntry` holds it.
#[derive(Clone, Debug, PartialEq)]
pub struct Word {
    pub id: i64,
    pub word: String,
    pub freq: i32,
    pub locale: Option<String>,
    pub shortcut: Option<String>,
}

/// A locale words are stored under, as Kotlin names it.
#[derive(Clone, Debug, PartialEq)]
pub struct Locale {
    pub stored: String,
    /// `displayName()`, which the list shows.
    pub label: String,
    /// `displayLanguage()`, which UserDictionaryScreen sorted by.
    pub language: String,
}

/// A `WORDS` report.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub dictionary: String,
    /// Whether the dictionary is on: Kotlin has no database for one that is off.
    pub on: bool,
    /// What the effect it answers did: a `SettingsUiWordOutcome`.
    pub outcome: String,
    /// In `queryAll()` order, which an export keeps.
    pub words: Vec<Word>,
    pub locales: Vec<Locale>,
}

/// A language on the first screen: its [`group`], name and `displayLanguage()`.
#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub key: Option<String>,
    pub label: String,
    pub language: String,
}

/// The user dictionary screens' own state.
#[derive(Debug, Default)]
pub struct Words {
    pub report: Option<Report>,
    /// The dictionary a report was asked for, until it comes.
    pub requested: Option<String>,
    /// The word list's search text.
    pub search: String,
    pub draft: Option<editor::Draft>,
    /// The dictionary an import or export works for, while the picker or the file work runs.
    pub transfer: Option<transfer::Transfer>,
}

/// A tap on a user dictionary screen's own controls.
#[derive(Clone, Debug, PartialEq)]
pub enum Tap {
    /// A language on the first screen opens its words.
    Group(Option<String>),
    Action(super::catalogue::words::Act),
    /// The editor's Delete.
    Delete,
}

/// Which dictionary and page `route` shows.
pub fn place(route: &str) -> Option<(&'static str, Page)> {
    let rest = route.strip_prefix(ROOT)?;
    let (d, rest) = [dictionary::FLORIS, dictionary::SYSTEM]
        .into_iter()
        .find_map(|d| Some((d, rest.strip_prefix(d)?)))?;
    let page = match rest {
        "" => Page::Languages,
        "/words" => Page::Words(None),
        "/word/add" => Page::Add,
        _ => match (
            rest.strip_prefix("/words/"),
            rest.strip_prefix("/word/edit/"),
        ) {
            (Some(key), _) => Page::Words(Some(key.into())),
            (_, Some(id)) => Page::Edit(id.parse().ok()?),
            _ => return None,
        },
    };
    Some((d, page))
}

/// The route of the screen that draws `route`.
pub fn screen_route(route: &str) -> Option<&'static str> {
    let (d, page) = place(route)?;
    let routes = if d == dictionary::FLORIS {
        [FLORIS, FLORIS_WORDS, FLORIS_ADD, FLORIS_EDIT]
    } else {
        [SYSTEM, SYSTEM_WORDS, SYSTEM_ADD, SYSTEM_EDIT]
    };
    Some(match page {
        Page::Languages => routes[0],
        Page::Words(_) => routes[1],
        Page::Add => routes[2],
        Page::Edit(_) => routes[3],
    })
}

/// The route of dictionary `d`'s `page`.
pub fn route(d: &str, page: &Page) -> String {
    match page {
        Page::Languages => format!("{ROOT}{d}"),
        Page::Words(None) => format!("{ROOT}{d}/words"),
        Page::Words(Some(key)) => format!("{ROOT}{d}/words/{key}"),
        Page::Add => format!("{ROOT}{d}/word/add"),
        Page::Edit(id) => format!("{ROOT}{d}/word/edit/{id}"),
    }
}

/// The language a word stored under `locale` is listed under in dictionary `d`, None for all
/// languages: what the Floris dictionary's converter reads as no locale (null, `all`, `null` or
/// empty), or the system's null. Otherwise the locale as `FlorisLocale.fromTag` reads it, its
/// language in lower case and country in upper, the old and new codes for Hebrew, Yiddish and
/// Indonesian taken as one, so `en-us` and `en_US` are one language.
pub fn group(d: &str, locale: Option<&str>) -> Option<String> {
    let s = locale?;
    if d == dictionary::FLORIS && matches!(s, "all" | "null" | "") {
        return None;
    }
    use wire::utilities::locale::{COUNTRY, LANGUAGE, VARIANT};
    let parts = utilities::locale::parse(&settings::text(s));
    let mut language = String::from_utf16_lossy(&parts[LANGUAGE]).to_ascii_lowercase();
    if let Some(new) = [("iw", "he"), ("ji", "yi"), ("in", "id")]
        .iter()
        .find(|(old, _)| language == *old)
    {
        language = new.1.into();
    }
    let country = String::from_utf16_lossy(&parts[COUNTRY]).to_ascii_uppercase();
    let tag = utilities::locale::tag(
        u16::from(b'_'),
        &settings::text(&language),
        &settings::text(&country),
        &parts[VARIANT],
    );
    Some(String::from_utf16_lossy(&tag))
}

/// A report's rows as a [`Report`]: the dictionary's row first, then words and locales.
pub fn parse(p: &[String]) -> Result<Report, &'static str> {
    if p.is_empty() || !p.len().is_multiple_of(row::SIZE) {
        return Err("settings words");
    }
    let mut r = Report::default();
    for (n, f) in p.chunks(row::SIZE).enumerate() {
        match (n, f[row::KIND].as_str()) {
            (0, kind::DICTIONARY) => {
                r.dictionary = f[row::ID].clone();
                r.on = f[row::WORD] == wire::flag::TRUE;
                r.outcome = f[row::LABEL].clone();
            }
            (0, _) | (_, kind::DICTIONARY) => return Err("settings words first"),
            (_, kind::WORD) => r.words.push(word(f)?),
            (_, kind::LOCALE) => r.locales.push(Locale {
                stored: f[row::LOCALE].clone(),
                label: f[row::LABEL].clone(),
                language: f[row::LANGUAGE].clone(),
            }),
            _ => return Err("settings words kind"),
        }
    }
    Ok(r)
}

/// `w` as an effect's row.
pub fn encode(w: &Word) -> Vec<String> {
    let mut f = vec![String::new(); row::SIZE];
    f[row::ID] = w.id.to_string();
    f[row::WORD] = w.word.clone();
    f[row::FREQ] = w.freq.to_string();
    f[row::LOCALE] = w.locale.clone().unwrap_or_default();
    f[row::HAS_LOCALE] = wire::flag::encode(w.locale.is_some()).into();
    f[row::SHORTCUT] = w.shortcut.clone().unwrap_or_default();
    f[row::HAS_SHORTCUT] = wire::flag::encode(w.shortcut.is_some()).into();
    f
}

/// The word a report's or effect's row holds.
fn word(f: &[String]) -> Result<Word, &'static str> {
    let flag = |i: usize| f[i] == wire::flag::TRUE;
    Ok(Word {
        id: f[row::ID].parse().map_err(|_| "settings words id")?,
        word: f[row::WORD].clone(),
        freq: f[row::FREQ].parse().map_err(|_| "settings words freq")?,
        locale: flag(row::HAS_LOCALE).then(|| f[row::LOCALE].clone()),
        shortcut: flag(row::HAS_SHORTCUT).then(|| f[row::SHORTCUT].clone()),
    })
}

/// The word an effect's row holds, as [`encode`] wrote it.
pub fn decode(f: &[String]) -> Option<Word> {
    (f.len() == row::SIZE).then(|| word(f).ok()).flatten()
}

impl App {
    /// A `WORDS` report: the dictionary as it is now, and how the change it answers went.
    pub fn set_words(&mut self, p: &[String]) -> Result<(), &'static str> {
        let report = parse(p)?;
        if self.words.requested.as_deref() == Some(report.dictionary.as_str()) {
            self.words.requested = None;
        }
        match report.outcome.as_str() {
            outcome::IMPORTED => self.toast(self.label("settings__udm__dictionary_import_success")),
            outcome::FAILED => self.toast(self.label("settings__udm__change_failed")),
            _ => {}
        }
        self.words.report = Some(report);
        // An editor reopened before the report came finds its word now.
        if let Some((_, page @ Page::Edit(_))) = self.word_place()
            && self.words.draft.is_none()
        {
            self.open_word(&page);
        }
        Ok(())
    }
    /// The dictionary and page on screen, if a user dictionary screen is.
    pub fn word_place(&self) -> Option<(&'static str, Page)> {
        place(self.stack.last()?)
    }
    /// The report, when it is the dictionary on screen's.
    pub fn word_report(&self) -> Option<&Report> {
        let (d, _) = self.word_place()?;
        self.words.report.as_ref().filter(|r| r.dictionary == d)
    }
    /// Asks Kotlin for dictionary `d` unless that report is on its way.
    pub(super) fn request_words(&mut self, d: &str) {
        if self.words.requested.as_deref() != Some(d) {
            self.words.requested = Some(d.into());
            self.effects.push(vec![effect::LOAD.into(), d.into()]);
        }
    }
    /// Pushes `kind` for the dictionary on screen with `rows`.
    pub(super) fn word_effect(&mut self, kind: &str, rows: &[&Word]) {
        let Some((d, _)) = self.word_place() else {
            return;
        };
        let mut packet = vec![kind.to_owned(), d.to_owned()];
        packet.extend(rows.iter().flat_map(|w| encode(w)));
        self.effects.push(packet);
    }
    /// The languages of the report on screen, as the first screen lists them: all languages
    /// first, then by `displayLanguage()` as UserDictionaryScreen sorted, each once.
    pub fn word_groups(&self) -> Vec<Group> {
        let Some(r) = self.word_report() else {
            return vec![];
        };
        let mut out: Vec<Group> = vec![];
        for w in &r.words {
            let key = group(&r.dictionary, w.locale.as_deref());
            if out.iter().any(|g| g.key == key) {
                continue;
            }
            let named = r
                .locales
                .iter()
                .find(|l| w.locale.as_deref() == Some(l.stored.as_str()));
            out.push(match (&key, named) {
                (None, _) => Group {
                    key,
                    label: self.label("settings__udm__all_languages"),
                    language: String::new(),
                },
                (Some(k), named) => Group {
                    key: key.clone(),
                    label: named.map_or(k.clone(), |l| l.label.clone()),
                    language: named.map_or(k.clone(), |l| l.language.clone()),
                },
            });
        }
        let units = |s: &str| s.encode_utf16().collect::<Vec<_>>();
        out.sort_by(|a, b| {
            (a.key.is_some(), units(&a.language)).cmp(&(b.key.is_some(), units(&b.language)))
        });
        out
    }
    /// The name of the language `key`: its first screen's label, or the key while no word
    /// is stored under it.
    pub fn group_label(&self, key: &Option<String>) -> String {
        let named = self.word_groups().into_iter().find(|g| g.key == *key);
        match (named, key) {
            (Some(g), _) => g.label,
            (None, Some(k)) => k.clone(),
            (None, None) => self.label("settings__udm__all_languages"),
        }
    }
    /// The top bar's title: a word list's is its language, as Gboard's is.
    pub fn word_title(&self) -> Option<String> {
        match self.word_place()? {
            (_, Page::Words(key)) => Some(self.group_label(&key)),
            _ => None,
        }
    }
    /// A native screen opened at `route`.
    pub(super) fn words_opened(&mut self, route: &str) {
        let page = place(route);
        if !matches!(page, Some((_, Page::Words(_)))) {
            self.forget_deleted(super::items::Source::Words);
        }
        if !matches!(page, Some((_, Page::Add | Page::Edit(_)))) {
            self.words.draft = None;
        }
        let Some((d, page)) = page else { return };
        // The first screen and the word list show the dictionary as it is now.
        if matches!(page, Page::Languages | Page::Words(_)) || self.words.report.is_none() {
            self.request_words(d);
        }
        if matches!(page, Page::Words(_)) {
            self.words.search.clear();
        }
        if matches!(page, Page::Add | Page::Edit(_)) {
            self.open_word(&page);
        }
    }
    /// Back went to the screen now on top.
    pub(super) fn words_left(&mut self) {
        let page = self.word_place().map(|(_, p)| p);
        if !matches!(page, Some(Page::Words(_))) {
            self.forget_deleted(super::items::Source::Words);
        }
        if !matches!(page, Some(Page::Add | Page::Edit(_))) {
            self.words.draft = None;
        }
    }
    /// After a saved state came back: the report again, and a draft if the editor lost it.
    pub fn words_restored(&mut self) {
        let Some((d, page)) = self.word_place() else {
            return;
        };
        self.words.requested = None;
        self.request_words(d);
        if matches!(page, Page::Add | Page::Edit(_)) && self.words.draft.is_none() {
            self.open_word(&page);
        }
    }
    /// A tap on a user dictionary screen's own control.
    pub(super) fn words_tap(&mut self, tap: Tap) {
        use super::catalogue::words::Act;
        let Some((d, _)) = self.word_place() else {
            return;
        };
        match tap {
            Tap::Group(key) => self.navigate(&route(d, &Page::Words(key))),
            Tap::Action(Act::Add) => self.navigate(&route(d, &Page::Add)),
            Tap::Action(Act::Import) => self.start_import(d),
            Tap::Action(Act::Export) => self.start_export(d),
            Tap::Action(Act::Manager) => self.word_effect(effect::MANAGER, &[]),
            Tap::Delete => self.delete_word(),
        }
    }
}
