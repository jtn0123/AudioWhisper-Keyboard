//! The extension screens, drawn by Rust at Compose's `ext` routes: ExtensionHomeScreen, the
//! theme, keyboard and language pack extension lists, an extension's page (ExtensionViewScreen,
//! or ExtensionNotFoundScreen when no extension has the id) and CheckUpdatesScreen. Kotlin
//! reports the extension index and deletes an extension (`wire::settings_ui::extensions`); Rust
//! picks what each screen lists, builds the update link and draws them (`boxes.rs`, `page.rs`).
//! The importer and Export are `import.rs`; editing an extension is still Compose's screen.
pub mod boxes;
pub mod chips;
mod dialogs;
pub mod import;
pub mod import_view;
pub mod page;
use super::{App, catalogue::languages::Lang};
use wire::settings_ui::extensions::{done, effect, kind, row, work};

pub const HOME: &str = "ext";
pub const THEMES: &str = "ext/list/EXT_THEME";
pub const KEYBOARDS: &str = "ext/list/EXT_KEYBOARD";
pub const PACKS: &str = "ext/list/EXT_LANGUAGEPACK";
/// An extension's page is `ext/view/<id>`.
pub const VIEW: &str = "ext/view";
pub const UPDATES: &str = "ext/check-updates";
/// ImportExtensionBox's Import: `Routes.Ext.Import(EXT_ANY, null)`.
pub const IMPORT: &str = import::ANY;
/// Visit Addons Store: `https://${BuildConfig.FLADDONS_STORE_URL}/`.
pub const STORE: &str = "https://beta.addons.florisboard.org/";
/// The store's host, which the update link names.
pub const STORE_HOST: &str = "beta.addons.florisboard.org";
/// The extension types' serial names, as the lists filter them.
pub const THEME: &str = "ime.extension.theme";
pub const KEYBOARD: &str = "ime.extension.keyboard";
pub const LANGUAGE_PACK: &str = "ime.extension.languagepack";
/// The icons the extension screens draw through a variable, which SettingsIcons must map
/// (`SettingsResourcesTest` reads this list).
pub const EXT_ICONS: [&str; 13] = [
    "palette",
    "keyboard",
    "language",
    "shop",
    "input",
    "file_download",
    "add",
    "info",
    "edit",
    "delete",
    "share",
    "mail",
    "link",
];
/// The strings the screens show beyond their titles.
pub const KEYS: [&str; 44] = [
    "ext__home__info",
    "ext__home__visit_store",
    "action__import",
    "ext__update_box__manual_updates_hint",
    "ext__update_box__search_for_updates",
    "ext__editor__title_create_any",
    "ext__list__view_details",
    "action__edit",
    "action__delete",
    "action__export",
    "ext__meta__maintainers",
    "ext__meta__id",
    "ext__meta__version",
    "ext__meta__keywords",
    "ext__meta__homepage",
    "ext__meta__issue_tracker",
    "ext__meta__license",
    "ext__meta__components_theme",
    "ext__meta__components_language_pack",
    "ext__meta__components_none_found",
    "ext__error__not_found_title",
    "ext__error__not_found_description",
    "ext__meta__email",
    "ext__meta__website",
    "action__delete_confirm_title",
    "action__delete_confirm_message",
    "action__select_files",
    "state__no_files_selected",
    "ext__meta__maintainers_by",
    "ext__import__file_skip",
    "ext__import__file_skip_unsupported",
    "ext__import__file_skip_ext_core",
    "ext__import__file_skip_ext_corrupted",
    "ext__import__success",
    "ext__import__failure",
    "ext__export__success",
    "ext__export__failure",
    "settings__ext__skip_other_type",
    "settings__ext__read_failed",
    "settings__ext__files_gone",
    "settings__ext__unwritable",
    "settings__udm__no_picker",
    "settings__files__interrupted",
    "error__snackbar_message",
];

/// A link as the page shows it: its host, and where it goes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Link {
    pub link: String,
    pub host: String,
}

/// An `ExtensionMaintainer`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Maintainer {
    pub name: String,
    pub email: Option<String>,
    pub url: Option<String>,
}

/// A theme or a language pack an extension bundles.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Component {
    pub id: String,
    pub label: String,
    /// A theme's stylesheet path, or a language pack's locale tag.
    pub detail: String,
    pub night: bool,
    pub authors: Vec<String>,
}

/// An installed extension, as its `ExtensionMeta` and components have it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Extension {
    pub id: String,
    /// The extension type's serial name.
    pub serial: String,
    pub title: String,
    pub version: String,
    pub license: String,
    /// Whether the extension manager can delete it: it was imported or made here.
    pub deletable: bool,
    pub description: Option<String>,
    pub homepage: Option<Link>,
    pub issue_tracker: Option<Link>,
    pub keywords: Vec<String>,
    pub maintainers: Vec<Maintainer>,
    pub components: Vec<Component>,
}

/// The extension screens' own state.
#[derive(Debug, Default)]
pub struct Extensions {
    /// The last report, in `combinedExtensionList()` order; None until one comes.
    pub list: Option<Vec<Extension>>,
    /// Kotlin was asked for the index; it reports again whenever the index changes.
    pub requested: bool,
    pub import: import::Import,
}

/// A tap on the extension screens' own controls.
#[derive(Clone, Debug, PartialEq)]
pub enum Tap {
    Store,
    Import,
    /// The importer's Select files.
    Select,
    /// Search for Updates.
    Update,
    Create,
    View(String),
    Edit(String),
    /// The page's Delete, asking first.
    Delete,
    Export,
    /// Maintainer `m`'s addresses, in a dialog.
    Maintainer(usize),
    /// The homepage or the issue tracker.
    Link(String),
}

/// An `EXTENSIONS` report: the extensions in their indexes' order, each with the rows after it.
pub fn parse(p: &[String]) -> Result<Vec<Extension>, &'static str> {
    if !p.len().is_multiple_of(row::SIZE) {
        return Err("settings extensions");
    }
    let mut out: Vec<Extension> = vec![];
    for f in p.chunks(row::SIZE) {
        let at = |i: usize| f[i].clone();
        if f[row::KIND] == kind::EXTENSION {
            out.push(Extension {
                id: at(row::EXTENSION),
                serial: at(row::ID),
                title: at(row::LABEL),
                version: at(row::DETAIL),
                license: at(row::TEXT),
                deletable: f[row::FLAG] == wire::flag::TRUE,
                ..Default::default()
            });
            continue;
        }
        let e = out
            .last_mut()
            .filter(|e| e.id == f[row::EXTENSION])
            .ok_or("settings extensions owner")?;
        let link = || {
            Some(Link {
                link: at(row::TEXT),
                host: at(row::DETAIL),
            })
        };
        match f[row::KIND].as_str() {
            kind::DESCRIPTION => e.description = Some(at(row::TEXT)),
            kind::HOMEPAGE => e.homepage = link(),
            kind::ISSUE_TRACKER => e.issue_tracker = link(),
            kind::KEYWORD => e.keywords.push(at(row::LABEL)),
            kind::MAINTAINER => e.maintainers.push(Maintainer {
                name: at(row::LABEL),
                ..Default::default()
            }),
            kind::EMAIL | kind::URL => {
                let m = e
                    .maintainers
                    .last_mut()
                    .ok_or("settings extensions maintainer")?;
                match f[row::KIND] == kind::EMAIL {
                    true => m.email = Some(at(row::TEXT)),
                    false => m.url = Some(at(row::TEXT)),
                }
            }
            kind::COMPONENT => e.components.push(Component {
                id: at(row::ID),
                label: at(row::LABEL),
                detail: at(row::DETAIL),
                night: f[row::FLAG] == wire::flag::TRUE,
                authors: vec![],
            }),
            kind::AUTHOR => {
                let c = e
                    .components
                    .last_mut()
                    .ok_or("settings extensions component")?;
                c.authors.push(at(row::LABEL));
            }
            _ => return Err("settings extensions kind"),
        }
    }
    Ok(out)
}

/// The update link for `list`, as `generateUpdateUrl` built it with `Uri.Builder`: the store's
/// `check-updates` page, its fragment the extensions' ids and versions as a JSON object, the
/// pairs joined by commas. Compose wrote a comma after each extension not `equals` to the last;
/// a keyboard extension is a data class, so one equal to the last lost its comma.
pub fn update_url<'a>(list: impl IntoIterator<Item = &'a Extension>) -> String {
    let pairs: Vec<String> = list
        .into_iter()
        .map(|e| format!("\"{}\":\"{}\"", e.id, e.version))
        .collect();
    format!(
        "https://{STORE_HOST}/check-updates#data={{{}}}",
        pairs.join(",")
    )
}

/// The catalogue route of the extension screen at `route`: a list's route without its query,
/// and `ext/view` for any extension's page.
pub fn screen_route(route: &str) -> Option<&'static str> {
    let base = route.split_once('?').map_or(route, |(base, _)| base);
    if base
        .strip_prefix("ext/view/")
        .is_some_and(|id| !id.is_empty())
    {
        return Some(VIEW);
    }
    [THEMES, KEYBOARDS, PACKS].into_iter().find(|r| *r == base)
}

/// Whether `route` asks a list for ImportExtensionBox and UpdateBox: `showUpdate=true`.
fn shows_update(route: &str) -> bool {
    route
        .split_once('?')
        .is_some_and(|(_, q)| q.split('&').any(|kv| kv == "showUpdate=true"))
}

/// What a component's box says about it, as ExtensionComponentView wrote it: Kotlin's list
/// `toString` of its authors, then a theme's night flag and stylesheet or a language pack's
/// locale, which ends in a newline.
pub fn component_text(serial: &str, c: &Component) -> String {
    let authors = format!("authors = [{}]\n", c.authors.join(", "));
    match serial {
        THEME => format!(
            "{authors}isNightTheme = {}\nstylesheetPath = {}",
            c.night, c.detail
        ),
        _ => format!("{authors}locale = {}\n", c.detail),
    }
}

/// Whether Kotlin's `isNullOrBlank` holds for `s`.
fn blank(s: &Option<String>) -> bool {
    s.as_ref()
        .is_none_or(|s| utilities::blank(&s.encode_utf16().collect::<Vec<u16>>()))
}

impl App {
    /// An `EXTENSIONS` report: the installed extensions as they are now.
    pub fn set_extensions(&mut self, p: &[String]) -> Result<(), &'static str> {
        self.extensions.list = Some(parse(p)?);
        Ok(())
    }
    /// An `EXTENSION_DONE` packet. A deleted extension's page closes, as Compose popped it.
    pub fn extension_done(&mut self, p: &[String]) -> Result<(), &'static str> {
        if p.len() != done::SIZE || p[done::WORK] != work::DELETE {
            return Err("settings extension done");
        }
        let gone = p[done::DONE] == wire::flag::TRUE;
        if gone && self.page_id() == Some(p[done::EXTENSION].as_str()) {
            self.back();
        }
        Ok(())
    }
    /// Every installed extension, as the last report gave them.
    pub fn extension_list(&self) -> &[Extension] {
        self.extensions.list.as_deref().unwrap_or(&[])
    }
    /// The id of the extension whose page is on top.
    pub fn page_id(&self) -> Option<&str> {
        let top = self.stack.last()?;
        (screen_route(top) == Some(VIEW)).then(|| &top[VIEW.len() + 1..])
    }
    /// The extension whose page is on top, as `getExtensionById` finds it: the first with the id.
    pub fn page_extension(&self) -> Option<&Extension> {
        let id = self.page_id()?;
        self.extension_list().iter().find(|e| e.id == id)
    }
    /// The serial name of the list on top.
    pub fn list_serial(&self) -> Option<&'static str> {
        match screen_route(self.stack.last()?)? {
            THEMES => Some(THEME),
            KEYBOARDS => Some(KEYBOARD),
            PACKS => Some(LANGUAGE_PACK),
            _ => None,
        }
    }
    /// The extensions the screen on top lists, or checks for updates: a list's type, or all.
    pub fn listed_extensions(&self) -> Vec<&Extension> {
        let serial = self.list_serial();
        self.extension_list()
            .iter()
            .filter(|e| serial.is_none_or(|s| e.serial == s))
            .collect()
    }
    /// Whether the screen on top shows ImportExtensionBox and UpdateBox: home always, a list
    /// when its route asks.
    pub fn shows_store(&self) -> bool {
        self.stack
            .last()
            .is_some_and(|r| r == HOME || shows_update(r))
    }
    /// The page's title: the extension's, the not-found title once the index says there is none,
    /// and nothing before the first report. None off the page.
    pub(super) fn ext_title(&self) -> Option<String> {
        self.page_id()?;
        if self.extensions.list.is_none() {
            return Some(String::new());
        }
        Some(match self.page_extension() {
            Some(e) => e.title.clone(),
            None => self.label("ext__error__not_found_title"),
        })
    }
    /// Whether the page shows the extension's homepage or issue tracker link.
    pub fn shows_link(link: &Option<Link>) -> bool {
        !blank(&link.as_ref().map(|l| l.link.clone()))
    }
    /// Asks Kotlin for the index unless it was asked: it reports every change after that.
    fn request_extensions(&mut self) {
        if !self.extensions.requested {
            self.extensions.requested = true;
            self.effects.push(vec![effect::LOAD.into()]);
        }
    }
    /// A native screen opened at `route`: an extension screen reads the index, and the importer
    /// its workspace.
    pub(super) fn extensions_opened(&mut self, route: &str) {
        if import::parts(route).is_some() {
            self.import_opened();
        } else if [HOME, UPDATES].contains(&route) || screen_route(route).is_some() {
            self.request_extensions();
        }
    }
    /// After a saved state came back: Kotlin's side is new, so the index is asked for again.
    pub fn extensions_restored(&mut self) {
        self.extensions = Extensions::default();
        if let Some(top) = self.stack.last().cloned() {
            self.extensions_opened(&top);
        }
    }
    /// A tap on an extension screen's own control.
    pub(super) fn ext_tap(&mut self, tap: Tap) {
        match tap {
            Tap::Store => self.effects.push(vec!["url".into(), STORE.into()]),
            Tap::Import => self.navigate(IMPORT),
            Tap::Select => self.select_files(),
            Tap::Update => {
                let url = update_url(self.listed_extensions());
                self.effects.push(vec!["url".into(), url]);
            }
            Tap::Create => self.navigate(super::themes::CREATE),
            Tap::View(id) => self.navigate(&format!("{VIEW}/{id}")),
            Tap::Edit(id) => self.navigate(&format!("ext/edit/{id}")),
            Tap::Delete => {
                if let Some(id) = self.page_extension().map(|e| e.id.clone()) {
                    self.open_lang_dialog(Lang::DeleteExtension, &id);
                }
            }
            Tap::Export => {
                if let Some(id) = self.page_id().map(str::to_owned) {
                    self.export(&id);
                }
            }
            Tap::Maintainer(m) => {
                if !self.addresses(m).is_empty() {
                    self.open_lang_dialog(Lang::Maintainer, &m.to_string());
                }
            }
            Tap::Link(url) => self.effects.push(vec!["url".into(), url]),
        }
    }
    /// Maintainer `m`'s addresses on the page, as its dialog lists them: `mailto:` and the email,
    /// then the website.
    pub fn addresses(&self, m: usize) -> Vec<String> {
        let Some(m) = self.page_extension().and_then(|e| e.maintainers.get(m)) else {
            return vec![];
        };
        let email = m.email.iter().map(|e| format!("mailto:{e}"));
        email.chain(m.url.iter().cloned()).collect()
    }
}
