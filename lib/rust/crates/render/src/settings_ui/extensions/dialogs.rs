//! The page's dialogs, drawn by the settings dialog on the page's hidden rows: the delete
//! question naming the extension (FlorisConfirmDeleteDialog), and a maintainer's addresses
//! (ExtensionMaintainerChip's dialog). Compose listed the addresses as chips with an icon; here
//! each is a list item with the icon, the address and what it is, which TalkBack reads.
use crate::panels::{Panel, rect};
use crate::settings_ui::{
    App,
    catalogue::{Kind, languages::Lang},
};
use keys::Rect;
use wire::settings_ui::extensions::effect;

impl App {
    /// The maintainer whose addresses the open dialog lists.
    pub fn maintainer_dialog(&self) -> Option<usize> {
        let m = self.modal.as_ref()?;
        match self.rows()[m.row].kind {
            Kind::Lang(Lang::Maintainer) => m.selected.parse().ok(),
            _ => None,
        }
    }
    /// The maintainer dialog's title: the maintainer's name.
    pub(in crate::settings_ui) fn maintainer_name(&self) -> Option<String> {
        let m = self.maintainer_dialog()?;
        Some(self.page_extension()?.maintainers.get(m)?.name.clone())
    }
    /// The delete question, naming the extension by its title as the page showed it.
    pub(in crate::settings_ui) fn ext_delete_message(&self, id: &str, template: &str) -> String {
        let title = self
            .extension_list()
            .iter()
            .find(|e| e.id == id)
            .map_or(id, |e| e.title.as_str());
        self.label(template).replace("{name}", title)
    }
    /// An address's label: the email without `mailto:`, or the website as written.
    pub(in crate::settings_ui) fn address_label(choice: &str) -> String {
        choice.strip_prefix("mailto:").unwrap_or(choice).to_owned()
    }
    /// What an address is: Email or Website.
    pub(in crate::settings_ui) fn address_kind(&self, choice: &str) -> String {
        self.label(match choice.starts_with("mailto:") {
            true => "ext__meta__email",
            false => "ext__meta__website",
        })
    }
    /// An address tapped: opened as the chip's `launchUrl` did, and the dialog closed.
    pub(in crate::settings_ui) fn address_chosen(&mut self, choice: String) {
        self.effects.push(vec!["url".into(), choice]);
    }
    /// The delete question's Delete: Kotlin deletes the extension and reports back.
    pub(in crate::settings_ui) fn delete_confirmed(&mut self, id: String) {
        self.effects.push(vec![effect::DELETE.into(), id]);
    }
    /// The address row `b`'s leading icon: 24 dp, 16 dp in, centred, in onSurfaceVariant.
    pub(in crate::settings_ui) fn address_icon(
        &mut self,
        p: &mut Panel,
        choice: &str,
        b: Rect,
        clip: Rect,
    ) {
        if self.maintainer_dialog().is_none() {
            return;
        }
        let size = self.px(24.0);
        let at = rect(
            b.left + self.px(16.0),
            b.top + ((b.height() - size) / 2.0).round(),
            size,
            size,
        );
        let name = match choice.starts_with("mailto:") {
            true => "mail",
            false => "link",
        };
        self.paint_icon(p, name, at, self.colors[2], clip);
    }
    /// The width an address row's text leaves: the icon's 56 dp lead and the 16 dp end.
    pub(in crate::settings_ui) fn address_insets(&self) -> [f32; 2] {
        [self.px(56.0), self.px(16.0)]
    }
}
