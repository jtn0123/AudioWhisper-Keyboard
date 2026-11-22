"""Languages and layouts (stage4-languages.md): each case must fail the test its comment names."""
from .paths import SETTINGS_UI, UI, LANG

CASES = [
    # Languages & Layouts (stage4-languages.md): each must fail SettingsLanguagesMatchTest, a route or resource test,
    # or a Rust language test.
    ('move-up-goes-down', UI / 'items.rs',
     'let to = if up { from.checked_sub(1)? } else { from + 1 };', 'let to = if up { from + 1 } else { from + 1 };'),
    ('undo-appends', LANG / 'list.rs', 'list.insert(gone.index.min(list.len()), item);', 'list.push(item);'),
    ('editor-checks-characters-only', LANG / 'editor.rs',
     'fields.push(field_text(v, &["layoutMap", key]));', 'fields.push(field_text(v, &["layoutMap", "characters"]));'),
    ('editor-adds-as-modify', LANG / 'editor.rs',
     'let op = if d.adding { "add" } else { "modify" };', 'let op = if d.adding { "modify" } else { "modify" };'),
    ('exists-closes-silently', LANG / 'editor.rs', 'None if d.adding => {', 'None if d.adding && d.id == i64::MIN => {'),
    ('custom-actions-include-off', UI / 'items.rs',
     '                if on {\n                    custom.push((id, label));\n                }', '                custom.push((id, label));'),
    ('language-route-dropped', UI / 'catalogue.rs', '    languages::PACKS,\n', ''),
    ('language-message-not-read', UI / 'dialog.rs',
     'if matches!(r.kind, Kind::Backup(_) | Kind::Lang(_)) {', 'if matches!(r.kind, Kind::Backup(_)) {'),
    ('language-string-id-missing', SETTINGS_UI / 'SettingsLanguageStringIds.kt',
     '    "settings__list__move_up" to R.string.settings__list__move_up,\n', ''),
    ('report-names-swapped', SETTINGS_UI / 'SettingsLanguages.kt',
     'SettingsUiLanguage.NATIVE to locale.displayName(locale), SettingsUiLanguage.ENGLISH to locale.displayName(FlorisLocale.ENGLISH)',
     'SettingsUiLanguage.NATIVE to locale.displayName(FlorisLocale.ENGLISH), SettingsUiLanguage.ENGLISH to locale.displayName(locale)'),
    ('search-tag-anywhere', LANG / 'picker.rs', '|| starts(&n.tag)', '|| has(&n.tag)'),
    ('native-names-ignored', LANG / 'mod.rs', 'if self.value(NAMES_KEY) == "NATIVE_LOCALE" {', 'if self.value(NAMES_KEY) == "NATIVE" {'),
    ('pack-deleted-without-asking', LANG / 'mod.rs',
     'self.open_lang_dialog(super::catalogue::languages::Lang::DeletePack, &id)',
     'self.effects.push(vec![wire::settings_ui::languages::effect::DELETE_PACK.into(), id])'),
    ('report-asked-every-screen', LANG / 'mod.rs', 'if !self.languages.requested {', 'if self.languages.requested || true {'),
    ('normalised-list-ignored', LANG / 'mod.rs', '.filter(|r| r.basis == raw);', '.filter(|r| r.basis == raw && r.basis.is_empty());'),
]
# The Rust test binary that must panic for each Rust-only case.
RUST = dict.fromkeys([
    'search-tag-anywhere', 'pack-deleted-without-asking',
], 'settings_ui_languages_picker') | dict.fromkeys([
    'native-names-ignored', 'report-asked-every-screen', 'normalised-list-ignored',
], 'settings_ui_languages')
