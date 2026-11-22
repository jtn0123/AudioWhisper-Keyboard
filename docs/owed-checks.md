# Device checks still owed

2026-11-21. Every device or phone check that the docs say is still owed, one entry each. Each
entry says what is owed, which device runs it, and the doc section that owes it. An entry is
removed only when a doc records it as run and passed.

This page does not run anything. It gathers what the docs say.

Two device names are used:

* **Phone** is the owner's Samsung S24 Ultra at 120 Hz.
* **Emulator** is the Android emulator the stage document names. Most use `aw_keyboard_ssd`.

## Confirm on the phone

These four come from the grade report. Where the linked doc does not state a phone check, the
row says so.

| # | What | Owed by |
|---|---|---|
| C1 | **E6.** The read-aloud fallback voice stays on the phone. Reading uses only a voice that needs no network and has its data installed. With none for the language, nothing is read. Check the phone's voice list, and a language with no offline voice. | [decisions: read-aloud's phone voice must work offline](decisions.md#read-alouds-phone-voice-must-work-offline-2026-11-16). The decision states the rule. It has no phone check. |
| C2 | **E7.** Each settings frame fits the screen's buffers before it is copied. Rotate and resize the Rust settings screen on the phone. A blank frame would mean the rule is wrong. | [decisions: settings frames copied only into buffers of their own size](decisions.md#settings-frames-copied-only-into-buffers-of-their-own-size-2026-11-16) |
| C3 | **E8.** The GPU drawing path is left out of the shipped library. Check that the phone's build draws the keyboard and settings with the CPU renderer. | [decisions: the GPU renderer is left out of every build that ships](decisions.md#the-gpu-renderer-is-left-out-of-every-build-that-ships-2026-11-16) |
| C4 | **G1.** How often a space or punctuation key reuses the strip's decode on the phone, and what a miss costs on the main thread. The host replay reused it at every word end (259/259); on the phone a fast space can arrive before the last letter's strip is decoded. Type at full speed with auto-correction on and count reuses and misses. | [decisions: the auto-commit reuses the strip's decode](decisions.md#the-auto-commit-reuses-the-strips-decode-when-nothing-has-changed-2026-11-16), which says a low hit rate would change it |

## Phone checks the docs owe

| # | What | Owed by |
|---|---|---|
| P1 | Keyboard: a real TalkBack session on the phone. Swipes follow the reading order. Touch exploration finds a held key's choices over the keys. The one-handed side panel has its place in the order. Right-to-left layouts still read left to right. | [compose-removal, K item 1](compose-removal.md#k-the-compose-keyboard) |
| P2 | Keyboard: the Samsung font check. | [compose-removal, K item 2](compose-removal.md#k-the-compose-keyboard); [rust.md, Milestone A](rust.md#milestone-a-prototype-2026-11-12) |
| P3 | Keyboard: font scale 2.0. | [compose-removal, K item 3](compose-removal.md#k-the-compose-keyboard) |
| P4 | Keyboard: warm scene draw p95 at most 2 ms at 120 Hz on the S24 Ultra. Run `cpu-trials.py` twice, remeasure tap positions for 1440x3120, and take a Perfetto trace. The emulator passed at 1.257 and 1.107 ms. | [compose-removal, K item 4](compose-removal.md#k-the-compose-keyboard); [decisions: the CPU gate](decisions.md#the-keyboards-cpu-gate-stays-2-ms-settled-on-the-phone-at-120-hz-2026-11-17) |
| P5 | Keyboard: the quick hide and show. Shown again within about a second of hiding, the keys must draw. The fix was checked on the emulator only. | [rust.md, shown again quickly](rust.md#shown-again-quickly-drawn-blank-fixed-2026-11-17); [stage4-other-theme, owed](stage4-other-theme.md#owed). [all-rust phase 1](all-rust.md#1-phone-checks-then-the-compose-keyboard-sm-waiting-on-the-owner) asks for it by hand. |
| P6 | Settings: a real TalkBack session on the phone, for each settings screen, before compose-removal step 12. | [stage4-followup2, item 5](stage4-followup2.md#still-owed-before-prototype-acceptance); [stage4-followup3](stage4-followup3.md#still-owed); [stage4-followup4](stage4-followup4.md#still-owed); [stage4-licences](stage4-licences.md#device-results) |
| P7 | Settings: the floating selection toolbar on a focused editor, and confirmation of predictive back and system animation. | [stage4-followup2, item 5](stage4-followup2.md#still-owed-before-prototype-acceptance) |
| P8 | Settings: the presenter breakdown (buffer wait, lock, copy, apply) on a device run. Confirm the CPU raster's 1.7–2.0 ms headroom at 120 Hz. | [stage4-followup3, A5](stage4-followup3.md#still-owed) |
| P9 | Other and Theme: every step ran on the emulator. None ran on the phone. | [stage4-other-theme, no phone check](stage4-other-theme.md#owed) |
| P10 | Emoji: pull the Samsung emoji font, inspect it, and render it with the same probe. The probe ran only on the other fonts. | [stage3a, system emoji fonts](stage3a-results.md#system-emoji-fonts) |
| P11 | Back up and Restore: every step ran on the emulator. None ran on the phone. | [stage4-backup-restore, owed](stage4-backup-restore.md#owed) |
| P12 | User dictionaries: every step ran on the emulator. None ran on the phone. | [stage4-user-dictionaries, owed](stage4-user-dictionaries.md#owed) |
| P13 | Keyboard: a top-row key's popup shows whole over the app. Long-press e, p and q with the Smartbar on and off, in portrait and landscape and in one-handed mode, and slide to a choice in the upper row. Tap the app just above the keyboard. Fixed and checked on the emulator, in portrait only. | [rust.md, a top-row key's popup](rust.md#a-top-row-keys-popup-cut-off-fixed-2026-11-21) |

## Emulator checks the docs owe

| # | What | Owed by |
|---|---|---|
| E1 | Restore the app prefs on the emulator with `SettingsDevice.cleanup`, then run `settings-device-baseline.py --verify`. The emulator stays in a fixture state until this runs. | [stage4-followup3, still owed](stage4-followup3.md#still-owed) |
| E2 | The whole instrumented suite on the final source: `ui` on both renderers with preference-byte parity, `text`, `death` and `crash`. The retained Compose path has 23 unlabeled controls to fix or explain. | [stage4-followup2, item 2](stage4-followup2.md#still-owed-before-prototype-acceptance); [stage4-followup3, A3](stage4-followup3.md#still-owed) |
| E3 | Final-source pictures: the 51 keyboard pairs and the group 1 gallery in light, dark and font scale 2, with the strict diff. | [stage4-followup2, item 1](stage4-followup2.md#still-owed-before-prototype-acceptance); [stage4-followup3, A2](stage4-followup3.md#still-owed) |
| E4 | Part B: the group 1 walk on both renderers, with the walk and preference comparison. | [stage4-followup3, still owed](stage4-followup3.md#still-owed) |
| E5 | The 23 Jetpref nodes. The classification has not run on a device, and fixing them needs a Jetpref fork. | [stage4-followup3, A4](stage4-followup3.md#still-owed) |
| E6 | The Smartbar's auto-expand row draws a checked, disabled switch. Compare its painted geometry with a Compose picture. | [stage4-followup3, still owed](stage4-followup3.md#still-owed) |
| E7 | The switch in a dialog title, at densities other than 2.625. | [stage4-followup4, still owed](stage4-followup4.md#still-owed) |
| E8 | Presented latency and jank: native SurfaceFrame and input-to-present timing, then a like-for-like comparison. Display-frame statistics from SurfaceFlinger are not enough. | [stage4-followup2, item 3](stage4-followup2.md#still-owed-before-prototype-acceptance); [stage4-settings-results, controlled baseline](stage4-settings-results.md#controlled-baseline-timing-and-memory) |
| E9 | Warm PSS and Perfetto on the final source, then the post-reopen baseline check. Paired PSS is below +10 MiB now, and no newer numbers exist. | [stage4-followup2, item 4](stage4-followup2.md#still-owed-before-prototype-acceptance) |
| E10 | Font scale 2.0 pixels: glyph ink about 0.12 px apart, and dialog buttons 1–3 px apart at the non-linear sizes. The owner decides legibility against parity. | [stage4-followup4, still owed](stage4-followup4.md#still-owed); [stage4-licences, device results](stage4-licences.md#device-results) |
| E11 | The project licence screen's text selection, with handles at the scroll edges. | [stage4-licences, text selection](stage4-licences.md#text-selection-project-licence-screen-owed) |
| E12 | Other: a device run of the dial's plural step, which looks for "1 hour" on the 24-hour dial, after `b4b47eb0e`. The emulator would not boot again. | [stage4-other-theme, owed](stage4-other-theme.md#owed) |
| E13 | TalkBack replay: the 400 ms input fixture failed (exit 1). The TalkBack settings route failed too, and TalkBack was never enabled on the emulator. Rerun on a working route. | [stage3b, TalkBack replay](stage3b-results.md#talkback-replay-and-limits); [stage3b-input-history, device boundary](stage3b-input-history.md#device-validation-boundary) |
| E14 | Repeat deadline: the 300 ms hold and the production repeat deadline on a device. The hooked APK never reached the device, so neither renderer has accepted this check. | [stage2-tts, deterministic long delete](stage2-tts.md#deterministic-long-delete) |
| E15 | Core package 12 on a device: PCM, cancellation, timing, the screenshot pair and the dictation replay. The old-build keyboard stop-key control is unresolved. | [stage2-tts, still owed](stage2-tts.md#still-owed--limits) |
| E16 | Final device hold on `aw_keyboard_ssd`: corrected final volume and DND readback, a literal `FATAL` or `AndroidRuntime` logcat scan, and `shutdown`. Five stage 2 documents share this wording. The TTS document has the full hold. | [stage2-input](stage2-input.md#final-fresh-device-gate-and-hold); [stage2-editor](stage2-editor.md#final-fresh-device-gate-and-hold); [stage2-dictation](stage2-dictation.md#final-fresh-device-gate-and-hold); [stage2-settings](stage2-settings.md#final-fresh-device-gate-and-hold); [stage2-backup-utils](stage2-backup-utils.md#final-fresh-device-gate-and-hold); [stage2-tts](stage2-tts.md#final-gate-and-device-hold) |
| E17 | Dictation: final prototype-off, Always day and Floris Day, accessibility and permission restoration. Not verified. The exact microphone and notification grant flags were seen earlier. | [stage2-dictation, final device hold](stage2-dictation.md#final-device-hold-and-recovery-state) |
| E18 | Settings device checks: Part A and backup/utils first; install with `scripts/install-dev.sh`; mixed synthetic settings and two subtypes; upgrade and compare; remove the active subtype through the UI; the existing fixtures on both renderers; the long-delete oracle; a clean `FATAL` scan; restore; `shutdown`. None of it ran. | [stage2-settings, owed device checks](stage2-settings.md#owed-device-checks) |
| E19 | Backup and utils: back up and restore through the real SAF UI, compare values and store hashes, check the incompatible-archive message against the pre-change build, and check MIME and locale on Android. Android URI round trips and ICU and device parity are also owed. Back up and restore through the system picker ran on the emulator for compose-removal step 5; the rest is still owed. | [stage2-backup-utils, owed device acceptance](stage2-backup-utils.md#owed-device-acceptance); [stage4-backup-restore, emulator check](stage4-backup-restore.md#emulator-check) |
| E20 | `SettingsOtherThemeDevice`: its Back up and Restore step now expects the Rust screens. It compiles but has not run. | [stage4-backup-restore, owed](stage4-backup-restore.md#owed) |
| E21 | Our keyboard typing into our own Rust settings fields. The fields take text: with agent-device's own test keyboard active, Select language's search took "germ" and filtered the list, and the home screen's preview field took "hello". With our keyboard active, agent-device refuses text entry into the keyboard's own app ("the focused input belongs to the active keyboard/IME"), and its taps on our keys typed nothing, so whether a person's taps type there is unchecked. Type into Select language's search, and a word into the user dictionary editor, by hand with our keyboard, on the phone or on an emulator. | [stage4-languages, emulator check](stage4-languages.md#emulator-check); [stage4-user-dictionaries, owed](stage4-user-dictionaries.md#owed) |
| E22 | A top-row key's popup in landscape without full-screen mode (set *landscape input* so the extract view never shows, then put it back) and in one-handed mode. The cargo tests cover both layouts; no device has shown them. | [rust.md, a top-row key's popup](rust.md#a-top-row-keys-popup-cut-off-fixed-2026-11-21) |

## Not device checks, but still owed

These need no device. They are listed so they are not lost.

* GIPHY live results need an API key. ([stage2-consolidation, open items](stage2-consolidation.md#open-items))
* The settings renderer has no route transition and no dialog enter animation. TalkBack's page scroll jumps where Compose animates it. ([stage4-licences, device results](stage4-licences.md#device-results))
* Copying the read-only hex value needs text selection, which waits for step 8. ([stage4-other-theme, owed](stage4-other-theme.md#owed))
* The English strings Jetpref and Material wrote are not translated. ([stage4-other-theme, owed](stage4-other-theme.md#owed))
* The clock-hand animation and Material's 100 ms pause are not matched. ([stage4-other-theme, owed](stage4-other-theme.md#owed))
* Focus highlights after a touch: whether a tap should hide them is a question for every Rust screen. ([stage4-other-theme, owed](stage4-other-theme.md#owed))
* The Rust edit nodes do not report focus, so tools that type through accessibility refuse them. ([stage4-other-theme, owed](stage4-other-theme.md#owed))
* One colour emoji still makes vello_cpu's 64 MiB atlas page resident. ([stage4-followup3, A6](stage4-followup3.md#still-owed))
* Writes queued when a Rust settings screen is destroyed are dropped. Nothing in the suite hit this. ([stage4-followup4, still owed](stage4-followup4.md#still-owed))
* A disabled row's node is clickable on Compose and not on Rust. Neither offers the click action. ([stage4-followup4, still owed](stage4-followup4.md#still-owed))
* Jetpref's store defect is upstream. `SettingsPreference` works around it. ([stage4-followup4, still owed](stage4-followup4.md#still-owed))
* A Merge restore cannot bring back a setting left at its default, because the backup does not hold it. ([stage4-backup-restore, owed](stage4-backup-restore.md#owed))
* A ZIP missing its folder entries reads as unreadable, in Restore and extension import. ([stage4-backup-restore, owed](stage4-backup-restore.md#owed))
* Restore runs on the main thread, as Compose's did. ([stage4-backup-restore, owed](stage4-backup-restore.md#owed))
* Compose's empty user dictionary route hands back to Rust's dictionary list, as it cannot tell the two dictionaries apart. Nothing opens that route now. ([stage4-user-dictionaries, owed](stage4-user-dictionaries.md#owed))
* The mutation scripts have no check that each case still matches its source. A renamed line makes a case match nothing, which stops a full run (`predictive-cancel-commits` did, from b57a98bf0 until it was fixed). A `--check` mode in each script, run by the gate, would catch it at once.
* In full-screen landscape a top-row key's popup is still cut at the keyboard's top: the frozen Compose root wraps the keyboard and clips its surface. The plain-View root must not clip it ([all-rust phase 3](all-rust.md#3-rust-owns-the-keyboard-l), item 4). ([rust.md, a top-row key's popup](rust.md#a-top-row-keys-popup-cut-off-fixed-2026-11-21))
* The Rust keyboard draws no key-press preview bubble. Its setting reaches only the Compose keyboard. ([rust.md, a top-row key's popup](rust.md#a-top-row-keys-popup-cut-off-fixed-2026-11-21))

## Notes

* This list has 4 confirm-on-the-phone items, 13 phone checks and 22 emulator checks.
