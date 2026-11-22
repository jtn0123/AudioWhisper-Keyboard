"""The extension editor (stage4-extensions.md, part 3): each case must fail the test its comment names."""
from .paths import EXT, SETTINGS_UI, UI

EDITOR = EXT / 'editor'
MOD = EDITOR / 'mod.rs'
WORK = EDITOR / 'work.rs'
DRAFT = EDITOR / 'draft.rs'
FORMS = EDITOR / 'forms.rs'

# Each must fail SettingsExtensionEditorMatchTest, SettingsFilesTest or SettingsResourcesTest.
JVM = [
    ('ext-editor-json-night-always', DRAFT, 'if !c.night {\n        out.insert("isNight".into(), json!(false));',
     'if true {\n        out.insert("isNight".into(), json!(c.night));'),
    ('ext-editor-json-empty-dependencies', DRAFT, 'if !self.dependencies.is_empty() {', 'if true {'),
    ('ext-editor-json-id-untrimmed', DRAFT, 'Ok(component(\n            &s(theme::ID),', 'Ok(component(\n            &self.id,'),
    ('ext-editor-json-blank-path-kept', DRAFT, 'let path = present.then(|| s(theme::STYLESHEET));',
     'let path = (present || !self.path.is_empty()).then(|| self.path.clone());'),
    ('ext-editor-json-none-written', DRAFT, 'if let Some(value) = value {\n                out.insert(key.into(), value);',
     'if value.is_some() || key == "homepage" {\n                out.insert(key.into(), value.unwrap_or(Value::Null));'),
    ('ext-editor-meta-never-refused', FORMS, 'out.first().is_some_and(|f| wire::flag::decode(f))', 'out.first().is_some()'),
    ('ext-editor-string-id-dropped', SETTINGS_UI / 'SettingsExtensionStringIds.kt',
     '    "settings__ext_editor__gone" to R.string.settings__ext_editor__gone,\n', ''),
    ('ext-editor-uncovered-not-told', SETTINGS_UI / 'SettingsFiles.kt',
     'uncovered(arrayOf(SettingsUiFiles.IDLE))', 'uncovered(arrayOf())'),
    # Against ReferenceExtensionEditor: ExtensionComponentView, CreateComponentScreen, ExtensionEditFilesScreen.
    ('ext-editor-box-name-trimmed', EDITOR / 'paint.rs',
     'let name = format!("{}:{}", self.extensions.editor.draft.meta.id, comp.id);',
     'let name = format!("{}:{}", self.extensions.editor.draft.meta.id, comp.id.trim());'),
    ('ext-editor-box-stylesheet-as-written', EDITOR / 'paint.rs', 'detail: comp.stylesheet(),', 'detail: comp.path.clone(),'),
    ('ext-editor-copy-suffix-from-zero', FORMS, 'let mut n = 1;', 'let mut n = 0;'),
    ('ext-editor-fonts-filter-short', MOD, '        "application/vnd.ms-fontobject",\n', ''),
]
# Each must fail the Rust tests in settings_ui_ext_editor.
ONLY_RUST = [
    # Routes, opening and Kotlin's reports.
    ('ext-editor-create-unknown', MOD, 'Some(draft::THEME) => Target::Create,', 'Some(draft::THEME) => Target::Unknown,'),
    ('ext-editor-unknown-creates', MOD, 'Some(_) => Target::Unknown,', 'Some(_) => Target::Create,'),
    ('ext-editor-reopened-resets', MOD, 'if target(route).is_none() || self.extensions.editor.route == route {',
     'if target(route).is_none() {'),
    ('ext-editor-restored-unasked', MOD, '(e.requested, e.load) = (false, Load::Waiting);', '(e.requested, e.load) = (true, Load::Waiting);'),
    ('ext-editor-restored-without-draft', MOD, 'packet[open::MANIFEST] = e.draft.raw_json();', 'packet[open::MANIFEST] = String::new();'),
    ('ext-editor-foreign-left-open', MOD,
     '                self.effects.push(vec![effect::CLOSE.into(), ws]);\n            }\n            return Ok(());',
     '                let _ = ws;\n            }\n            return Ok(());'),
    ('ext-editor-any-kind-read', MOD, 'if head[row::KIND] != kind::WORKSPACE {', 'if false {'),
    ('ext-editor-unthemed-replaces', MOD, '} else if head[row::FLAG] == wire::flag::TRUE {', '} else if true {'),
    ('ext-editor-themed-unchanged', MOD, 'e.draft = Draft::parse(text)?;\n            e.changed = true;',
     'e.draft = Draft::parse(text)?;\n            e.changed = false;'),
    ('ext-editor-opened-unsynced', MOD, 'if fresh {\n            self.ext_sync();', 'if false {\n            self.ext_sync();'),
    ('ext-editor-taken-up-synced', MOD, 'if fresh {\n            self.ext_sync();', 'if true {\n            self.ext_sync();'),
    ('ext-editor-change-unsynced', MOD, 'self.extensions.editor.changed = true;\n        self.ext_sync();',
     'self.extensions.editor.changed = true;'),
    ('ext-editor-fresh-id-fixed', DRAFT, 'id: format!("local.themes.{now}"),', 'id: "local.themes".into(),'),
    # Leaving: Back, the question, and the workspace closing with the editor.
    ('ext-editor-back-leaves-save', MOD, 'if self.ext_editor_ready() && saving {', 'if false {'),
    ('ext-editor-back-never-asks', MOD, 'let ask = self.ext_editor_ready() && self.extensions.editor.changed;', 'let ask = false;'),
    ('ext-editor-discard-asks-again', MOD, 'self.extensions.editor.changed = false;\n        self.back();', 'self.back();'),
    ('ext-editor-left-open', MOD,
     '                    self.effects.push(vec![effect::CLOSE.into(), ws]);\n                }\n                self.extensions.editor = ExtEditor::default();',
     '                    let _ = ws;\n                }\n                self.extensions.editor = ExtEditor::default();'),
    # The editor's controls.
    ('ext-editor-delete-ignored', MOD, '                    self.extensions.editor.draft.themes.remove(i);\n',
     '                    let _ = i;\n'),
    ('ext-editor-delete-unchanged', MOD, 'self.extensions.editor.draft.themes.remove(i);\n                    self.ext_changed();',
     'self.extensions.editor.draft.themes.remove(i);'),
    ('ext-editor-theme-first-always', MOD, '.push(vec![effect::THEME.into(), ws, i.to_string()]);',
     '.push(vec![effect::THEME.into(), ws, "0".into()]);'),
    ('ext-editor-picker-fonts-only', MOD, 'let mime = MIME[f.min(1)];', 'let mime = MIME[0];'),
    ('ext-editor-from-inverted', MOD, 'c.empty = !existing;', 'c.empty = existing;'),
    # Save.
    ('ext-editor-invalid-meta-saved', MOD, 'if !forms::meta_valid(&e.draft.meta) {', 'if false {'),
    ('ext-editor-blank-component-unnamed', MOD, 'let name = if blank { c.label.clone() } else { c.id.clone() };',
     'let name = c.id.clone();'),
    ('ext-editor-create-takes-installed', MOD, '_ if e.installed.contains(&id) => {', '_ if false => {'),
    ('ext-editor-save-while-saving', MOD, 'if self.files.busy.is_some() || !self.editing() {', 'if !self.editing() {'),
    ('ext-editor-edit-untargeted', MOD, 'Some(Target::Edit(id)) => id,\n            _ if',
     'Some(Target::Edit(_)) => String::new(),\n            _ if'),
    ('ext-editor-done-only-when-open', WORK,
     'let mine = on && !e.workspace.is_empty() && e.workspace == p[done::NAME];',
     'let mine = on && self.editing() && e.workspace == p[done::NAME];'),
    ('ext-editor-saved-still-changed', WORK, '(e.workspace, e.changed) = (String::new(), false);',
     '(e.workspace, e.changed) = (String::new(), true);'),
    ('ext-editor-gone-save-kept', WORK, 'if outcome == result::GONE && mine {', 'if false {'),
    ('ext-editor-taken-vague', WORK, 'result::TAKEN => "settings__ext_editor__id_taken",',
     'result::TAKEN => "settings__ext_editor__save_failed",'),
    ('ext-editor-lost-save-vague', WORK, 'job::EXT_SAVE => "settings__ext_editor__save_interrupted",',
     'job::EXT_SAVE => "settings__files__interrupted",'),
    # Components: Create component and its copy.
    ('ext-editor-copy-failure-silent', WORK, 'return self.toast(self.label("settings__ext_editor__copy_failed"));',
     'return;'),
    ('ext-editor-copy-unchanged', WORK, 'self.extensions.editor.draft.themes.push(comp);\n        self.ext_changed();',
     'self.extensions.editor.draft.themes.push(comp);'),
    ('ext-editor-taken-id-allowed', FORMS, 'if k == 0 && !checked.2 && taken {', 'if false {'),
    ('ext-editor-empty-without-flags', FORMS, 'flags: self.extensions.editor.defaults.clone(),', 'flags: Default::default(),'),
    ('ext-editor-copy-same-id', FORMS, 'id: unique_id(&e.draft.themes, &src.comp.id),', 'id: src.comp.id.clone(),'),
    ('ext-editor-own-copy-from-archive', FORMS, 'if same { String::new() } else { src.extension },', 'src.extension,'),
    ('ext-editor-create-needs-nothing', FORMS, 'self.files.busy.is_none() && if c.empty { typed } else { chosen }',
     'self.files.busy.is_none()'),
    # The meta data sheet.
    ('ext-editor-installed-id-typed', FORMS, 'Edit::Meta(0) if self.editor_target() != Some(super::Target::Create) => None,',
     'Edit::Meta(0) if false => None,'),
    ('ext-editor-refused-meta-applied', FORMS, 'if refused {\n            f.checked = true;', 'if false {\n            f.checked = true;'),
    ('ext-editor-noop-apply-changes', FORMS, 'if meta != x.draft.meta {', 'if true {'),
    # The archive's files.
    ('ext-editor-name-leaves-folder', FORMS, "!name.is_empty() && !name.contains(['/', '\\0']) && name != \".\"",
     '!name.is_empty() && name != "."'),
    ('ext-editor-name-taken-allowed', FORMS, '(other && taken).then_some("settings__ext_editor__file_exists")', 'None'),
    ('ext-editor-added-untrimmed', FORMS, 'None => self.name.trim().into(),', 'None => self.name.clone(),'),
    ('ext-editor-same-name-renamed', FORMS, 'Some(old) if *old == name => self.close_editor(),',
     'Some(old) if old.is_empty() => self.close_editor(),'),
    ('ext-editor-add-any-type', FORMS, '(work::TEXT, MIME[f.folder.min(1)].join("|")),', '(work::TEXT, String::new()),'),
    ('ext-editor-renamed-kept', WORK, 'files.retain(|f| *f != gone);', 'let _ = gone;'),
    ('ext-editor-deleted-relisted', WORK, 'if kind != job::EXT_FILE_DELETE\n            && let Some(new) = split(name)',
     'if let Some(new) = split(name)'),
    ('ext-editor-wrong-type-vague', WORK, 'result::WRONG_TYPE => "settings__ext_editor__wrong_type",',
     'result::WRONG_TYPE => "settings__ext_editor__file_failed",'),
    # Layout and TalkBack.
    ('ext-editor-edit-unfocused', EDITOR / 'paint.rs', '[COMP_ID + 8 * c + 3, COMP_ID + 8 * c + 4]', '[COMP_ID + 8 * c + 3, COMP_ID + 8 * c + 3]'),
    ('ext-editor-bar-shown-waiting', UI / 'backup_rows.rs', ' && self.visible(i)\n', ' && true\n'),
    # Recreation.
    ('ext-editor-saved-unchanged', EDITOR / 'saved.rs', '"changed": e.changed,', '"changed": false,'),
    ('ext-editor-restored-anywhere', EDITOR / 'saved.rs', 'if target(&route).is_none() || workspace.is_empty() {',
     'if workspace.is_empty() {'),
    ('ext-editor-list-below-unasked', EXT / 'mod.rs', 'if self.stack.iter().any(|r| lists(r)) {', 'if false {'),
    ('ext-editor-editor-lists', EXT / 'mod.rs', '!own && ([HOME, UPDATES]', '([HOME, UPDATES]'),
    # A settings link mid-edit.
    ('ext-editor-link-replaces-edit', UI / 'mod.rs', 'if self.ext_editor_kept(route, root) {', 'if false {'),
    ('ext-editor-root-link-replaces', MOD, '        root || self\n', '        false || self\n'),
    ('ext-editor-lower-link-truncates', MOD, '.is_some_and(|p| p < at)', '.is_some_and(|p| p > at)'),
    ('ext-editor-second-editor-opened', MOD, '(other && self.open_edit().is_some()).then(|| e.route.clone())',
     '(other && false).then(|| e.route.clone())'),
]
CASES = JVM + ONLY_RUST
# The Rust test binary that must panic for each Rust-only case.
RUST = dict.fromkeys([case[0] for case in ONLY_RUST], 'settings_ui_ext_editor')
