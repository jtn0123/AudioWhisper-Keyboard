"""Settings, subtypes and backup (the app family's Match tests). settings-mutations.py,
subtype-mutations.py and backup-utils-mutations.py cover the same code through their own tests;
these cases are the ones the frozen Kotlin references used to catch."""
SETTINGS = 'lib/rust/crates/settings/src/'
CATALOGUE = SETTINGS + 'catalogue.tsv'
BACKUP = 'lib/rust/crates/backup/src/lib.rs'
EDITOR = 'lib/rust/crates/render/src/settings_ui/languages/editor.rs'
MODULE = ':app'
TARGETS = {
    'migration': ['*SettingsMigrationMatchTest'],
    'codec': ['*SettingsCodecMatchTest', '*SettingsNumericMatchTest'],
    'defaults': ['*SettingsDefaultsMatchTest'],
    'stored': ['*SettingsStoredFileMatchTest', '*SettingsUiStoredFileMatchTest'],
    'subtypes': ['*SettingsSubtypeMatchTest'],
    'languages': ['*SettingsLanguagesMatchTest'],
    'backup': ['*BackupPlanMatchTest', '*BackupArchiveMatchTest', '*BackupFilesTest', '*SettingsBackupFlowTest'],
}
CASES = [
    # Migration: renamed keys, reset keys, rewritten values and the action arrangement.
    ('theme-renamed-wrong', SETTINGS + 'migration.rs', '"advanced__settings_theme" => target = "other__settings_theme",',
     '"advanced__settings_theme" => target = "other__settings_themes",', 'migration'),
    ('one-handed-end-reset', SETTINGS + 'migration.rs', '(k == "keyboard__one_handed_mode" && raw == text("OFF"))',
     '(k == "keyboard__one_handed_mode" && raw == text("END"))', 'migration'),
    ('devtools-prefix-narrowed', SETTINGS + 'migration.rs', 'let reset = k.starts_with("devtools__")',
     'let reset = k.starts_with("devtools__e")', 'migration'),
    ('rgba-read-as-hex', SETTINGS + 'migration.rs', 'value = text(if raw == text("RGBA") { "RGB" } else { "HEX" });',
     'value = text(if raw == text("HEX") { "RGB" } else { "HEX" });', 'migration'),
    ('sync-false-kept', SETTINGS + 'migration.rs', '} else if raw == text("false") {', '} else if raw == text("OFF") {',
     'migration'),
    ('emoji-split-on-comma', SETTINGS + 'migration.rs', 'raw.split(|&u| u == 59)', 'raw.split(|&u| u == 44)', 'migration'),
    ('compact-not-replaced', SETTINGS + 'migration.rs', 'item = action(codes::TOGGLE_COMPACT_LAYOUT)?;',
     'item = action(codes::COMPACT_LAYOUT_TO_RIGHT)?;', 'migration'),
    ('forward-delete-not-added', SETTINGS + 'migration.rs', '        codes::FORWARD_DELETE,\n', '', 'migration'),
    ('popup-keys-deduplicated', SETTINGS + 'migration.rs', '        && data.get("popup").ok() == Some(&Value::Null)\n', '',
     'migration'),
    ('text-actions-kept-twice', SETTINGS + 'migration.rs',
     'Some(&Value::string("insert_text")) {\n        return true;', 'Some(&Value::string("insert_text")) {\n        return false;',
     'migration'),
    # Stored codecs: the JSON number lexer, unknown history fields, omitted defaults, Int range.
    ('exponent-sign-ignored', SETTINGS + 'numeric.rs', '} else if unit == 45 && has_exponent {\n            positive_exponent = false;',
     '} else if unit == 45 && has_exponent {\n            positive_exponent = true;', 'codec'),
    ('quoted-numbers-refused', SETTINGS + 'numeric.rs', 'let input = if input.first() == Some(&34) {',
     'let input = if input.first() == Some(&35) {', 'codec'),
    ('history-unknown-field-allowed', SETTINGS + 'codec.rs', 'k != "pinned" && k != "recent")',
     'k != "pinned" && k != "recent" && k != "unknown")', 'codec'),
    ('symbols2-layout-default', SETTINGS + 'codec.rs', '    ("symbols2", "western"),', '    ("symbols2", "western2"),', 'codec'),
    ('int-range-unchecked', SETTINGS + 'codec.rs', '            i32::try_from(n).map_err(|_| "integer overflow")?;\n', '',
     'codec'),
    # The catalogue's defaults and slider bounds, and the stored scalar and time readers.
    ('number-row-on', CATALOGUE, 'keyboard__number_row\tb\tfalse', 'keyboard__number_row\tb\ttrue', 'defaults'),
    ('long-press-minimum', CATALOGUE, 'keyboard__long_press_delay\ti\t300\t100\t', 'keyboard__long_press_delay\ti\t300\t101\t',
     'defaults'),
    ('history-size-step', CATALOGUE, 'clipboard__history_size_limit\ti\t20\t5\t100\t5\t',
     'clipboard__history_size_limit\ti\t20\t5\t100\t4\t', 'defaults'),
    ('sunrise-a-millisecond-late', CATALOGUE, 'theme__sunrise_time\ts\t06:00:00.000', 'theme__sunrise_time\ts\t06:00:00.001',
     'defaults'),
    ('long-minus-ignored', SETTINGS + 'scalar.rs', '        45 => (true, rest),', '        45 => (false, rest),', 'defaults'),
    ('false-read-as-true', SETTINGS + 'scalar.rs', '} else if raw == text("false") {\n                Some(text("false"))',
     '} else if raw == text("false") {\n                Some(text("true"))', 'defaults'),
    ('time-dot-refused', SETTINGS + 'time.rs', 'raw[5] != 58 || raw[8] != 46', 'raw[5] != 58 || raw[8] != 58', 'defaults'),
    ('time-millis-two-digits', SETTINGS + 'time.rs', '"{:02}:{:02}:{:02}.{:03}"', '"{:02}:{:02}:{:02}.{:02}"', 'stored'),
    # Subtype edits: the active fallback, duplicates on add, unsupported subtypes, cycling, modify.
    ('active-without-fallback', SETTINGS + 'subtype.rs', '.or(if ids.is_empty() { None } else { Some(0) });', '.or(None);',
     'subtypes'),
    ('add-lets-duplicates-in', SETTINGS + 'subtype.rs', '    b.set("id", Value::Number(0))?;', '    b.set("id", Value::Number(1))?;',
     'subtypes'),
    ('han-spelling-supported', SETTINGS + 'subtype.rs', '        || providers.get("spelling")? == &han\n', '', 'subtypes'),
    ('cycle-wraps-to-last', SETTINGS + 'subtype.rs', 'selected = order[0] as i64;', 'selected = order[order.len() - 1] as i64;',
     'subtypes'),
    ('modify-unchanged', SETTINGS + 'subtype.rs', '                list[index] = item.clone();\n                changed = true;\n',
     '                list[index] = item.clone();\n', 'subtypes'),
    # The language editor's draft check.
    ('suggestion-provider-unchecked', EDITOR, '        fields.push(field_text(v, &["nlpProviders", "suggestion"]));\n', '',
     'languages'),
    ('unset-locale-accepted', EDITOR, 'field_text(v, &["primaryLocale"]) != LOCALE_UNSET && fields.iter()',
     'field_text(v, &["primaryLocale"]) != LOCALE_UNSET || fields.iter()', 'languages'),
    # Backup and restore: metadata warnings and errors, the effect plan, the manifest, selection.
    ('vendor-prefix-shortened', BACKUP, 'starts_with(&text("dev.patrickgold.florisboard"))',
     'starts_with(&text("dev.patrickgold"))', 'backup'),
    ('version-64-refused', BACKUP, '|| version < 64 {', '|| version <= 64 {', 'backup'),
    ('only-empty-package-blank', BACKUP, 'if package.iter().all(|&u| chars::is_whitespace(u))', 'if package.is_empty()', 'backup'),
    ('clear-only-when-present', BACKUP, '        if erase {\n            step(ops::CLEAR_DIRECTORY',
     '        if erase && present(group) {\n            step(ops::CLEAR_DIRECTORY', 'backup'),
    ('incoming-inverted', BACKUP, '.filter(|(_, v)| !flag::decode(v))', '.filter(|(_, v)| flag::decode(v))', 'backup'),
    ('manifest-group-shifted', BACKUP, 'row[manifest::GROUP] = text(&group.to_string());',
     'row[manifest::GROUP] = text(&(group + 1).to_string());', 'backup'),
    ('clipboard-never-mixed', BACKUP, '} else if clips == offered {', '} else if clips != 0 {', 'backup'),
]
