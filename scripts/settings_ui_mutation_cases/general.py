"""The renderer, settings groups 1 and 2, Jetpref writes and the third-party licences: each case must fail the test its comment names."""
from .paths import ROOT, BASE, CAT, LIC, SETTINGS

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
     'back::CANCELLED => a.back_progress = 0.0,', 'back::CANCELLED => a.back(),'),
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
]
# The Rust test binary that must panic for each Rust-only case.
RUST = dict.fromkeys([
    'no-stretch', 'fling-not-absorbed', 'one-line-switch-56', 'disabled-switch-faded-enabled',
    'radio-text-half-pixel', 'switch-half-pixel-track', 'linked-divider-inside',
], 'settings_ui_overscroll') | dict.fromkeys([
    'confirm-generic-buttons', 'field-minimum-outside-padding', 'card-content-onsurface',
    'title-medium-published-tracking', 'card-icon-box-48dp',
], 'settings_ui_group_two') | dict.fromkeys([
    'advance-linear-round',
], 'android_text') | dict.fromkeys([
    'every-row-drawn',
], 'settings_ui_licences') | dict.fromkeys([
    'talkback-page-fraction',
], 'settings_ui_screens')
