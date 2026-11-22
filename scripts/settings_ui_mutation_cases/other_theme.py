"""Other and Theme (stage4-other-theme.md), with the colour and time pickers: each case must fail the test its comment names."""
from .paths import ROOT, CAT, UI

CASES = [
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
]
# The Rust test binary that must panic for each Rust-only case.
RUST = dict.fromkeys([
    'time-drag-rounds-down',
], 'settings_ui_time') | dict.fromkeys([
    'dial-moves-on-under-accessibility', 'drag-without-slop-trail', 'keyboard-mode-forgotten',
    'time-default-button-keeps', 'dial-minutes-read-as-hours',
], 'settings_ui_theme') | dict.fromkeys([
    'hex-label-zero-line',
], 'settings_ui_other_theme')
