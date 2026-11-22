//! Which theme is shown, and what Android loads to show it: `ThemeManager.updateActiveTheme`.
//!
//! A theme that loaded is kept, as a slot Android holds the loaded theme in, until the theme
//! extensions change. A theme that can't be loaded (its extension is a keyboard extension's id,
//! has no archive, or has no such component) changes nothing, so the last theme stays. A theme
//! whose stylesheet doesn't load shows the base theme, and is tried again next time.

use crate::Text;
use crate::choose::{Facts, Name};

/// A theme extension, as Android indexed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Extension {
    pub id: Text,
    /// It has an archive to load.
    pub source: bool,
    pub components: Vec<Component>,
}

/// One of an extension's themes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Component {
    pub id: Text,
    /// The stylesheet path it names; empty, or only spaces, if none.
    pub stylesheet: Text,
}

/// The extensions a theme is looked up in, as `ExtensionManager.getExtensionById` looks: a
/// keyboard extension with the id comes first, and isn't a theme. Language packs come after
/// themes, so they never decide a theme and aren't listed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Index {
    pub keyboards: Vec<Text>,
    pub themes: Vec<Extension>,
}

/// What Android does next.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Show the theme the editor previews.
    Preview,
    /// Show loaded theme `slot`.
    Cached(u32),
    /// Change nothing.
    Keep,
    /// Load the stylesheet at `path` in `index.themes[extension]`'s archive, as its component
    /// `component`, then report with [`Session::loaded`].
    Load {
        extension: usize,
        component: usize,
        path: Text,
    },
    /// The load worked: show it as loaded theme `slot`.
    Active(u32),
    /// The load failed: show the base theme.
    Failed,
}

/// The loaded themes and the load in progress.
#[derive(Debug, Default)]
pub struct Session {
    cache: Vec<(Name, u32)>,
    next: u32,
    pending: Option<Name>,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    /// The step that shows the theme `facts` choose, after dropping every loaded theme if
    /// `clear`. `preview_info` is whether the editor previews a theme it holds. A load not yet
    /// reported is abandoned.
    pub fn update(
        &mut self,
        clear: bool,
        preview_info: bool,
        facts: &Facts,
        index: &Index,
    ) -> Step {
        self.pending = None;
        if clear {
            self.cache.clear();
        }
        if preview_info {
            return Step::Preview;
        }
        let name = facts.active();
        if let Some((_, slot)) = self.cache.iter().find(|(cached, _)| cached == name) {
            return Step::Cached(*slot);
        }
        if index.keyboards.contains(&name.extension) {
            return Step::Keep;
        }
        let Some(extension) = index.themes.iter().position(|e| e.id == name.extension) else {
            return Step::Keep;
        };
        let theme = &index.themes[extension];
        let Some(component) = theme.components.iter().position(|c| c.id == name.component) else {
            return Step::Keep;
        };
        if !theme.source {
            return Step::Keep;
        }
        let stylesheet = &theme.components[component].stylesheet;
        let path = extensions::stylesheet_path(&name.component, stylesheet);
        self.pending = Some(name.clone());
        Step::Load {
            extension,
            component,
            path,
        }
    }

    /// What shows once the load the last step asked for worked (`ok`) or failed; `None` if no
    /// load is in progress.
    pub fn loaded(&mut self, ok: bool) -> Option<Step> {
        let name = self.pending.take()?;
        if !ok {
            return Some(Step::Failed);
        }
        let slot = self.next;
        self.next += 1;
        self.cache.push((name, slot));
        Some(Step::Active(slot))
    }

    /// How many loaded themes are kept.
    pub fn cached(&self) -> usize {
        self.cache.len()
    }
}
