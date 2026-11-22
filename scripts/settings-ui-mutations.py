"""Require fresh JVM assertion failures for each UI, editor, group 1, group 2, licences, Other, Theme, language and
user dictionary mutation, and fresh Rust test panics for each mutation that only Rust tests see; always restore bytes."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent
BASE = ROOT / 'lib/rust/crates'
CAT = BASE / 'render/src/settings_ui/catalogue'
LIC = BASE / 'render/src/settings_ui/licences'
SETTINGS = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/settings'
UI = BASE / 'render/src/settings_ui'
LANG = UI / 'languages'
WORDS = UI / 'words'
SETTINGS_UI = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/settingsui'
SYNC_CLEAR, SYNC_SET = ('    (\n        "ONLY_{0}_EVENTS",\n        "enum__clipboard_sync_behavior__only_{1}_events",\n'
                         '        "enum__clipboard_sync_behavior__only_{1}_events__description",\n    ),\n'
                         .format(event, event.lower()) for event in ('CLEAR', 'SET'))
CASES = [
    ('toggle', BASE / 'render/src/settings_ui/events.rs',
     'self.write(r.key, (!self.yes(r.key)).to_string())', 'self.write(r.key, self.yes(r.key).to_string())'),
    ('default-stays-open', BASE / 'render/src/settings_ui/events.rs',
     'Action::Default => {\n                if let Some(m) = self.modal.take()',
     'Action::Default => {\n                if let Some(m) = self.modal.clone()'),
    ('surrogate-delete', BASE / 'render/src/settings_ui/editor.rs',
     'right += if pair { 2 } else { 1 };', 'right += if pair { 1 } else { 1 };'),
    ('predictive-cancel-commits', BASE / 'awnative/src/render_settings.rs',
     '2 => a.back_progress = 0.0,', '2 => a.back(),'),
    # Group 1: each must fail SettingsGroupOneMatchTest or SettingsResourcesTest.
    ('condition', CAT / 'clipboard.rs', '        &MINUTES,\n    )\n    .on(All(&[\n        HISTORY,\n'
     '        Yes("clipboard__history_auto_clean_old_enabled"),\n    ])),', '        &MINUTES,\n    )\n    .on(HISTORY),'),
    ('entry-order', CAT / 'clipboard.rs', SYNC_CLEAR + SYNC_SET, SYNC_SET + SYNC_CLEAR),
    ('no-preview', CAT / 'input_feedback.rs', '    hardware: 1,\n    preview: true,', '    hardware: 1,\n    preview: false,'),
    ('confirm-swap', CAT / 'media.rs', 'Kind::Confirm(&RESET_PINNED)', 'Kind::Confirm(&RESET_RECENT)'),
    ('visible-fixed', CAT / 'smartbar.rs', '    .shown(SHARED),', '    .shown(Any(&[SHARED, EXTENDED])),'),
    # The fixed row is Compose's plain text: no button role, and no disabled state to report.
    ('fixed-row-button', BASE / 'render/src/settings_ui/layout.rs',
     'Kind::Info | Kind::Status(_) | Kind::Fixed => "text",', 'Kind::Info | Kind::Status(_) => "text",'),
    ('fixed-row-reported-disabled', BASE / 'render/src/settings_ui/mod.rs',
     'self.enabled || self.role == "text"', 'self.enabled'),
    ('stale-string-id', ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/settingsui/SettingsStringIds.kt',
     '    "action__yes" to R.string.action__yes,\n', ''),
    ('route-dropped', BASE / 'render/src/settings_ui/catalogue.rs', '    media::SCREEN,\n    typing', '    typing'),
    # The group 1 gallery's differences (stage4-followup4.md, part 1): each must fail a Rust test.
    ('no-stretch', BASE / 'render/src/settings_ui/overscroll.rs',
     '        let left = self.relax(delta);', '        let left = delta;'),
    ('fling-not-absorbed', BASE / 'render/src/settings_ui/overscroll.rs',
     'if velocity != 0.0 && self.stretch.edges[toward] > 0.0 {', 'if velocity != 0.0 && toward > 2 {'),
    ('one-line-switch-56', BASE / 'render/src/settings_ui/layout.rs',
     '                                64.0\n', '                                56.0\n'),
    ('disabled-switch-faded-enabled', BASE / 'render/src/settings_ui/material.rs',
     'match (enabled, checked) {', 'match (true, checked) {'),
    ('radio-text-half-pixel', BASE / 'render/src/settings_ui/dialog_metrics.rs',
     '[self.px(52.0), self.px(16.0)]', '[52.0 * self.config[2], 16.0 * self.config[2]]'),
    ('switch-half-pixel-track', BASE / 'render/src/settings_ui/material.rs',
     '[self.px(52.0), self.px(32.0), self.px(48.0)]', '[52.0 * self.config[2], 32.0 * self.config[2], self.px(48.0)]'),
    ('linked-divider-inside', BASE / 'render/src/settings_ui/material.rs',
     'b.left - 1.0,', 'b.left,'),
    # Group 2: each must fail SettingsGroupTwoMatchTest, SettingsEditorMatchTest or a Rust test.
    ('spell-any-package', BASE / 'render/src/settings_ui/rows.rs',
     '&& self.fact("spell_package") == self.fact("package")', '&& true'),
    ('spell-intent-swapped', BASE / 'render/src/settings_ui/events.rs',
     '"intent".into(), "spellCheckerSettings".into()', '"intent".into(), "hardKeyboardSettings".into()'),
    ('forget-no-toast', CAT / 'typing.rs', 'toast: "pref__suggestion__forget_learned__done",', 'toast: "",'),
    ('auto-space-card-always', CAT / 'typing.rs', '.shown(Yes("correction__auto_space_punctuation"))', '.shown(Always)'),
    ('changelog-version-unfilled', BASE / 'render/src/settings_ui/events.rs',
     '.replace("{version}", self.fact("version_name"))', '.replace("{version}", "{version}")'),
    ('physical-fact-inverted', CAT / 'physical_keyboard.rs',
     '.shown(Fact("physical_keyboard", "true"))', '.shown(Fact("physical_keyboard", "false"))'),
    ('models-never-checking', BASE / 'render/src/settings_ui/fields.rs',
     '"ready" if self.fact("models_checked") != "true"', '"ready" if false'),
    ('field-not-saved', BASE / 'render/src/settings_ui/fields.rs',
     'if self.prefs.get(key) != Some(&value) {', 'if false {'),
    ('field-enter-finishes', BASE / 'awnative/src/render_settings.rs', '66 if a.field.is_some() =>', '66 if false =>'),
    ('dictation-route-dropped', BASE / 'render/src/settings_ui/catalogue.rs', '    dictation::SCREEN,\n', ''),
    ('confirm-generic-buttons', BASE / 'render/src/settings_ui/dialog_buttons.rs',
     '["", ask.dismiss, ask.confirm]', '["", "action__no", "action__yes"]'),
    ('field-minimum-outside-padding', BASE / 'render/src/settings_ui/fields.rs',
     '.max(self.px(120.0) - 2.0 * v)', '.max(self.px(120.0))'),
    # The group 2 gallery's differences: each must fail a Rust test.
    ('card-content-onsurface', BASE / 'render/src/settings_ui/cards.rs',
     'BODY_LARGE, self.colors[13], clip', 'BODY_LARGE, self.colors[1], clip'),
    ('title-medium-published-tracking', BASE / 'render/src/settings_ui/cards.rs', 'tracking: 0.2,', 'tracking: 0.15,'),
    ('card-icon-box-48dp', BASE / 'render/src/settings_ui/cards.rs',
     'let padded = 2.0 * self.px(12.0) + self.px(24.0);', 'let padded = self.px(48.0);'),
    ('advance-linear-round', BASE / 'render/src/android_text.rs',
     'hinted.unwrap_or_else(|| pixel_round(base * k))', 'hinted.unwrap_or_else(|| (base * k).round())'),
    # Jetpref's store dies when a write's caller is cancelled mid-write: each must fail
    # SettingsUiWriteCancellationMatchTest.
    ('write-cancellable', SETTINGS / 'SettingsScalars.kt',
     'withContext(NonCancellable) { data.set(value) }', 'data.set(value)'),
    ('reset-cancellable', SETTINGS / 'SettingsScalars.kt',
     'withContext(NonCancellable) { data.reset() }', 'data.reset()'),
    ('custom-unwrapped', SETTINGS / 'SettingsDefaults.kt',
     '        SettingsPreference(custom(key, default, serializer))', '        custom(key, default, serializer)'),
    # Third-party licences: each must fail SettingsLicencesMatchTest, or a Rust test where named.
    ('licence-set-file-order', LIC / 'data.rs', '    order.sort();\n', '    order.sort_by_key(|o| o.1);\n'),
    ('library-order-code-points', LIC / 'data.rs', 'a.encode_utf16().cmp(b.encode_utf16())', 'a.cmp(b)'),
    ('author-organisation-first', LIC / 'data.rs', 'let author = if developers.is_empty() {',
     'let author = if organisation.is_some() || developers.is_empty() {'),
    ('blank-version-shown', LIC / 'list.rs', '.filter(|v| !blank(v))', '.filter(|v| !v.is_empty())'),
    ('html-legacy-block-lines', LIC / 'html.rs', '            self.newlines(1);\n            self.marks += 1;',
     '            self.newlines(2);\n            self.marks += 1;'),
    ('empty-licence-text-node', LIC / 'dialog.rs', '        if !body.is_empty() {\n', '        if !body.is_empty() || true {\n'),
    ('every-row-drawn', LIC / 'list.rs', '            if top >= clip.bottom {\n                break;\n            }\n', ''),    # TalkBack pages a list by the scroll node's own height, as Compose does: must fail a Rust test.
    ('talkback-page-fraction', BASE / 'render/src/settings_ui/events.rs',
     'self.set_scroll(self.scroll() + direction * page);', 'self.set_scroll(self.scroll() + direction * page * 0.6);'),
    # Other and Theme (stage4-other-theme.md): each must fail SettingsOtherThemeMatchTest or
    # SettingsUiMatchTest, or the Rust test named below.
    ('theme-day-enabled-by-night', CAT / 'theme.rs',
     '.on(Cond::Not("theme__mode", "ALWAYS_NIGHT"))', '.on(Cond::Not("theme__mode", "ALWAYS_DAY"))'),
    ('times-follow-system', CAT / 'theme.rs',
     'Cond::Is("theme__mode", "FOLLOW_TIME")', 'Cond::Is("theme__mode", "FOLLOW_SYSTEM")'),
    ('theme-no-preview', CAT / 'theme.rs', '    preview: true,\n    rows: ROWS,', '    preview: false,\n    rows: ROWS,'),
    ('addon-list-without-update', CAT / 'theme.rs',
     'route: "ext/list/EXT_THEME?showUpdate=true",', 'route: "ext/list/EXT_THEME",'),
    ('backup-opens-restore', CAT / 'other.rs',
     'Kind::Route("settings/other/backup")', 'Kind::Route("settings/other/restore")'),
    ('theme-route-dropped', UI / 'catalogue.rs', '    theme::SCREEN,\n', ''),
    ('theme-label-not-looked-up', UI / 'rows.rs', 'format!("theme_label:{id}")', 'format!("theme:{id}")'),
    ('theme-id-any-colons', UI / 'rows.rs', "stored.matches(':').count() == 1", "stored.contains(':')"),
    ('theme-label-facts-misnamed', ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/settingsui/SettingsFacts.kt',
     'listOf("theme_label:$id", label)', 'listOf("theme:$id", label)'),
    ('addon-title-not-lowercased', UI / 'addons.rs', '&self.label(a.title).to_lowercase()', '&self.label(a.title)'),
    ('time-hour-padded', UI / 'time.rs', 'format!("{h}:{minute:02}")', 'format!("{h:02}:{minute:02}")'),
    ('time-noon-am', UI / 'time.rs', 'if hour >= 12 { "PM" } else { "AM" }', 'if hour > 12 { "PM" } else { "AM" }'),
    ('time-stored-milliseconds', UI / 'time.rs',
     'format!("{hour:02}:{minute:02}:00.000")', 'format!("{hour:02}:{minute:02}:00.001")'),
    ('time-enter-on-hour-finishes', UI / 'time_events.rs', 'if key == 66 && j == 1 {', 'if key == 66 {'),
    ('accent-stored-as-colour', UI / 'color.rs', '    if argb == system_accent {', '    if argb == 0 {'),
    ('time-drag-rounds-down', UI / 'time.rs',
     '(angle / TAU * f32::from(marks) + 0.5).floor()', '(angle / TAU * f32::from(marks)).floor()'),
    ('dial-moves-on-under-accessibility', UI / 'time_events.rs',
     '        p.minutes |= autoswitch;\n        self.time_changed(was);', '        p.minutes = true;\n        self.time_changed(was);'),
    ('drag-without-slop-trail', UI / 'time_events.rs',
     'let (dx, dy) = (x - sx - centre.0, y - sy - centre.1);', 'let (dx, dy) = (x - centre.0 + 0.0 * sx, y - centre.1 + 0.0 * sy);'),
    ('keyboard-mode-forgotten', UI / 'time_events.rs', '                        self.time_inputs.push(key);\n', ''),
    ('time-default-button-keeps', UI / 'time_events.rs',
     '        if confirm {\n            self.write(key, stored);\n        } else {\n            self.reset(key);\n        }',
     '        if confirm {\n            self.write(key, stored);\n        }'),
    # The bug the step 4 pictures found: a 12 sp line height table miss, the label zero lines tall.
    ('hex-label-zero-line', UI / 'color_dialog.rs', 'self.metrics(BODY_SMALL)[0].ceil()', 'self.line_height(self.sp(12)).ceil()'),
    # The minutes read with the hour plural: "15 hours", or a bare 15 when Kotlin sent none.
    ('dial-minutes-read-as-hours', UI / 'time_dial.rs', '(true, _) => Some(PLURALS[1].0),', '(true, _) => Some(PLURALS[0].0),'),
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
    # User dictionaries (stage4-user-dictionaries.md): each must fail SettingsWordsMatchTest, SettingsWordsTest,
    # SettingsResourcesTest or a Rust user dictionary test.
    ('word-frequency-untrimmed', WORDS / 'editor.rs',
     'chars::text::to_int_or_null(chars::trim(&settings::text(&self.fields[1])))?',
     'chars::text::to_int_or_null(&settings::text(&self.fields[1]))?'),
    ('word-duplicate-allowed', WORDS / 'editor.rs', 'None if k == 0 && self.word_exists(d) =>',
     'None if k == 0 && self.word_exists(d) && d.fields[0].is_empty() =>'),
    ('word-locale-not-normalised', SETTINGS_UI / 'SettingsWords.kt',
     'entry.copy(locale = entry.locale?.let { FlorisLocale.fromTag(it).localeTag() })', 'entry'),
    ('word-import-not-upserted', SETTINGS_UI / 'SettingsWords.kt',
     'if (existing.isNotEmpty()) dao.update(entry.copy(id = existing[0].id)) else dao.insert(entry.copy(id = 0))',
     'dao.insert(entry.copy(id = 0))'),
    ('word-import-not-in-transaction', SETTINGS_UI / 'SettingsWords.kt',
     '{ transaction { words.forEach { importWord(dao, it) } }; return', '{ words.forEach { importWord(dao, it) }; return'),
    ('word-undo-new-id', SETTINGS_UI / 'SettingsWords.kt', 'dao.insert(if (taken) entry.copy(id = 0) else entry)', 'dao.insert(entry.copy(id = 0))'),
    ('word-import-null-locale-kept', WORDS / 'transfer.rs',
     'locale = some().filter(|v| v != "all" && v != "null");', 'locale = some().filter(|v| v != "all");'),
    ('word-import-partial', WORDS / 'transfer.rs',
     '.map(|(n, line)| entry(line).ok_or(n + 1))', '.filter_map(|(_, line)| entry(line).map(Ok))'),
    ('word-import-line-off-by-one', WORDS / 'transfer.rs', '.map(|(n, line)| entry(line).ok_or(n + 1))', '.map(|(n, line)| entry(line).ok_or(n))'),
    ('word-export-null-locale-empty', WORDS / 'transfer.rs', 'w.locale.as_deref().unwrap_or("null")', 'w.locale.as_deref().unwrap_or("")'),
    ('word-export-not-reread', WORDS / 'transfer.rs', '        self.words.requested = None;\n        self.request_words(d);\n', ''),
    ('word-delete-sends-restore', WORDS / 'list.rs', 'self.word_effect(effect::DELETE, &[&w]);', 'self.word_effect(effect::RESTORE, &[&w]);'),
    ('word-files-to-backup', UI / 'files.rs', '&& !self.words_picked(&purpose, &answer)', '&& !self.words_picked("-", &answer)'),
    ('word-string-id-missing', SETTINGS_UI / 'SettingsWordStringIds.kt',
     '    "settings__udm__no_picker" to R.string.settings__udm__no_picker,\n', ''),
    ('word-list-reorders', WORDS / 'list.rs', '    reorder: false,', '    reorder: true,'),
    ('list-paints-move-for-words', UI / 'items.rs', 'let first = if words.reorder { 0 } else { 2 };', 'let first = 0;'),
    ('list-focus-ignores-flag', UI / 'items.rs',
     '[reorder && i > 0, reorder && i + 1 < n, true]', '[reorder || i > 0, reorder || i + 1 < n, true]'),
    ('word-search-ignores-shortcut', WORDS / 'list.rs',
     '.filter(|w| holds(&w.word) || w.shortcut.as_deref().is_some_and(holds))', '.filter(|w| holds(&w.word))'),
    ('word-empty-card-under-search', UI / 'items.rs',
     'Source::Words => !self.group_has_words(),', 'Source::Words => self.word_items().is_empty(),'),
    ('word-added-without-language', WORDS / 'editor.rs',
     'Some((_, Page::Words(Some(key)))) => key,', 'Some((_, Page::Words(Some(_)))) => String::new(),'),
    ('word-title-not-language', WORDS / 'mod.rs',
     '(_, Page::Words(key)) => Some(self.group_label(&key)),', '(_, Page::Words(_)) => None,'),
    ('word-frequency-text-keyboard', WORDS / 'editor.rs',
     '.is_some_and(|r| matches!(r.kind, Kind::Words(Part::Field(1))))', '.is_some_and(|r| matches!(r.kind, Kind::Words(Part::Field(9))))'),
    ('word-undo-outlives-list', WORDS / 'mod.rs',
     '        let page = self.word_place().map(|(_, p)| p);\n        if !matches!(page, Some(Page::Words(_))) {\n'
     '            self.forget_deleted(super::items::Source::Words);\n        }\n', '        let page = self.word_place().map(|(_, p)| p);\n'),
    ('word-editor-delete-keeps-focus', WORDS / 'editor.rs',
     '        self.close_editor();\n        self.focus = crate::settings_ui::items::UNDO_ID;\n', '        self.close_editor();\n'),
    ('word-saved-search-dropped', WORDS / 'saved.rs', '        self.words.search = search;\n', '        self.words.search = search.replace(&search, "");\n'),
    ('word-title-forgets-name', WORDS / 'mod.rs', 'self.words.names.get(k).unwrap_or(k).clone()', 'k.clone()'),
]
# The Rust test binary that must panic for each Rust-only case.
RUST = dict.fromkeys([
    'no-stretch',
    'fling-not-absorbed',
    'one-line-switch-56',
    'disabled-switch-faded-enabled',
    'radio-text-half-pixel',
    'switch-half-pixel-track',
    'linked-divider-inside',
], 'settings_ui_overscroll') | dict.fromkeys([
    'dial-moves-on-under-accessibility',
    'drag-without-slop-trail',
    'keyboard-mode-forgotten',
    'time-default-button-keeps',
    'dial-minutes-read-as-hours',
], 'settings_ui_theme') | {'hex-label-zero-line': 'settings_ui_other_theme'} | {'time-drag-rounds-down': 'settings_ui_time'} | dict.fromkeys([
    'confirm-generic-buttons',
    'field-minimum-outside-padding',
    'card-content-onsurface',
    'title-medium-published-tracking',
    'card-icon-box-48dp',
], 'settings_ui_group_two') | dict.fromkeys(['search-tag-anywhere', 'pack-deleted-without-asking'], 'settings_ui_languages_picker') | dict.fromkeys([
    'native-names-ignored',
    'report-asked-every-screen',
    'normalised-list-ignored',
], 'settings_ui_languages') | dict.fromkeys([
    'word-list-reorders',
    'list-paints-move-for-words',
    'list-focus-ignores-flag',
    'word-search-ignores-shortcut',
    'word-empty-card-under-search',
    'word-added-without-language',
    'word-title-not-language',
    'word-frequency-text-keyboard',
    'word-undo-outlives-list',
    'word-editor-delete-keeps-focus',
    'word-saved-search-dropped',
    'word-title-forgets-name',
], 'settings_ui_words') | {'advance-linear-round': 'android_text', 'every-row-drawn': 'settings_ui_licences',
                            'talkback-page-fraction': 'settings_ui_screens'}


def run_rust(output, name, target):
    with (output / (name + '.log')).open('wb') as log:
        result = subprocess.run(['cargo', 'test', '-q', '-p', 'render', '--test', target],
                                cwd=ROOT / 'lib/rust', stdout=log, stderr=subprocess.STDOUT)
    text = (output / (name + '.log')).read_text(errors='replace')
    failures = [{'suite': target, 'type': 'panic'} for line in text.splitlines() if 'panicked at' in line]
    return result.returncode, failures


def run(output, name):
    if name in RUST:
        return run_rust(output, name, RUST[name])
    started = time.time_ns()
    with (output / (name + '.log')).open('wb') as log:
        result = subprocess.run(['./gradlew', ':app:testDebugUnitTest', '--tests', '*SettingsEditorMatchTest',
                                 '--tests', '*SettingsUi*MatchTest', '--tests', '*SettingsGroupOneMatchTest', '--tests', '*SettingsGroupTwoMatchTest',
                                 '--tests', '*SettingsLicencesMatchTest', '--tests', '*SettingsOtherThemeMatchTest',
                                 '--tests', '*SettingsResourcesTest', '--tests', '*SettingsLanguagesMatchTest',
                                 '--tests', '*SettingsWords*Test'], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
    failures = []
    for path in (ROOT / 'app/build/test-results/testDebugUnitTest').glob('TEST-*settingsui.Settings*Test.xml'):
        if path.stat().st_mtime_ns < started:
            continue
        suite = ET.parse(path).getroot()
        for failure in suite.findall('.//failure'):
            if 'AssertionFailedError' in failure.attrib.get('type', ''):
                failures.append({'suite': suite.attrib['name'], 'type': failure.attrib['type']})
    return result.returncode, failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('cases', nargs='*', help='case names to run (default: all)')
    args = parser.parse_args()
    unknown = set(args.cases) - {name for name, _, _, _ in CASES}
    assert not unknown, f'unknown cases: {sorted(unknown)}'
    cases = [case for case in CASES if not args.cases or case[0] in args.cases]
    args.output.mkdir(parents=True, exist_ok=False)
    results = []
    originals = {path: path.read_bytes() for _, path, _, _ in cases}
    try:
        for name, path, old, new in cases:
            source = path.read_text()
            assert source.count(old) == 1, name
            path.write_text(source.replace(old, new))
            try:
                status, failures = run(args.output, name)
                assert status != 0 and failures, name + ' must fail fresh differential assertions'
                results.append({'mutation': name, 'exit': status, 'fresh_assertions': len(failures)})
            finally:
                path.write_bytes(originals[path])
            print(name + ': caught', flush=True)
    finally:
        for path, original in originals.items():
            path.write_bytes(original)
    restored, failures = run(args.output, 'restored')
    assert restored == 0 and not failures
    for target in sorted({RUST[name] for name, _, _, _ in cases if name in RUST}):
        restored, failures = run_rust(args.output, 'restored-' + target, target)
        assert restored == 0 and not failures, target
    report = {'mutations': results, 'restored_exit': restored,
              'source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in originals}}
    (args.output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report), flush=True)


if __name__ == '__main__':
    main()
