"""The theme manager (stage4-theme-editor.md, part 1): each case must fail the test its comment names."""
from .paths import SETTINGS_UI, THEMES

CASES = [
    # The theme manager (compose-removal step 9, stage4-theme-editor.md): each JVM case must fail
    # SettingsThemesMatchTest or SettingsResourcesTest, each Rust case the Rust test named below.
    ('theme-sort-by-code-point', THEMES / 'mod.rs',
     '.sort_by(|a, b| a.label.encode_utf16().cmp(b.label.encode_utf16()));', '.sort_by(|a, b| a.label.cmp(&b.label));'),
    ('theme-repeat-keeps-old-themes', THEMES / 'mod.rs', '                        out[i] = ext;\n',
     '                        out[i].title = ext.title;\n'),
    ('theme-day-writes-night', THEMES / 'mod.rs', 'DAY => Some("theme__day_theme_id"),', 'DAY => Some("theme__night_theme_id"),'),
    ('theme-value-slash', THEMES / 'mod.rs', 'let value = format!("{}:{}", ext.id, theme.id);',
     'let value = format!("{}/{}", ext.id, theme.id);'),
    ('theme-pick-not-previewed', THEMES / 'mod.rs', 'self.write(key, value);\n        self.sync_theme_preview();',
     'self.write(key, value);'),
    ('theme-night-dropped', THEMES / 'mod.rs', 'night: flag(row::NIGHT),', 'night: false,'),
    ('theme-chosen-not-selected', THEMES / 'manager.rs', 'c.selected = chosen;', 'c.selected = false;'),
    ('theme-row-night-unsent', SETTINGS_UI / 'SettingsThemes.kt', 'SettingsUiTheme.NIGHT to WireFlag.encode(night)',
     'SettingsUiTheme.NIGHT to WireFlag.encode(false)'),
    ('theme-string-id-dropped', SETTINGS_UI / 'SettingsThemeStringIds.kt',
     '    "ext__list__view_details" to R.string.ext__list__view_details,\n', ''),
    ('theme-asked-every-open', THEMES / 'mod.rs', 'if !self.themes.requested {', 'if true {'),
    ('theme-report-keeps-asked', THEMES / 'mod.rs',
     '        self.themes.extensions = Some(parse(p)?);\n        self.themes.requested = false;\n',
     '        self.themes.extensions = Some(parse(p)?);\n'),
    ('theme-preview-every-sync', THEMES / 'mod.rs', 'if wanted != self.themes.preview {', 'if true {'),
    ('theme-back-keeps-preview', THEMES / 'mod.rs',
     '    pub(super) fn themes_left(&mut self) {\n        self.sync_theme_preview();\n    }', '    pub(super) fn themes_left(&mut self) {}'),
    ('theme-restore-not-asked', THEMES / 'mod.rs', 'if self.theme_key().is_some() {\n            self.request_themes();',
     'if false {\n            self.request_themes();'),
    ('theme-edit-for-built-ins', THEMES / 'manager.rs', 'if ext.deletable {\n            let edit', 'if true {\n            let edit'),
    ('theme-view-opens-editor', THEMES / 'mod.rs', 'Tap::View(id) => self.navigate(&format!("ext/view/{id}")),',
     'Tap::View(id) => self.navigate(&format!("ext/edit/{id}")),'),
    ('theme-import-creates', THEMES / 'mod.rs', 'Tap::Import => self.navigate(IMPORT),', 'Tap::Import => self.navigate(CREATE),'),
    ('theme-icon-unfaded', THEMES / 'manager.rs', 'let faded = (self.colors[2] & 0x00ff_ffff) | 0x8f00_0000;',
     'let faded = self.colors[2];'),
    ('theme-focus-skips-themes', THEMES / 'manager.rs',
     '            out.extend((0..ext.themes.len()).map(|t| theme_id(e, 4 + t)));\n', ''),
    ('theme-unknown-kind-accepted', THEMES / 'mod.rs', '_ => return Err("settings themes kind"),', '_ => {}'),
    ('theme-extension-unchecked', THEMES / 'mod.rs', 'if out[e].id != f[row::EXTENSION] {', 'if false {'),
]
# The Rust test binary that must panic for each Rust-only case.
RUST = dict.fromkeys([
    'theme-asked-every-open', 'theme-preview-every-sync', 'theme-back-keeps-preview', 'theme-restore-not-asked',
    'theme-edit-for-built-ins', 'theme-view-opens-editor', 'theme-import-creates', 'theme-icon-unfaded',
    'theme-focus-skips-themes', 'theme-unknown-kind-accepted', 'theme-extension-unchecked',
], 'settings_ui_theme_manager') | dict.fromkeys([
    'theme-report-keeps-asked',
], 'settings_ui_theme')
