"""Require fresh JVM assertion failures for each UI, editor, group 1 and group 2 mutation, and
fresh Rust test panics for each mutation that only Rust tests see; always restore bytes."""
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
SETTINGS = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/settings'
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
    'confirm-generic-buttons',
    'field-minimum-outside-padding',
    'card-content-onsurface',
    'title-medium-published-tracking',
    'card-icon-box-48dp',
], 'settings_ui_group_two') | {'advance-linear-round': 'android_text'}


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
                                 '--tests', '*SettingsResourcesTest'], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
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
