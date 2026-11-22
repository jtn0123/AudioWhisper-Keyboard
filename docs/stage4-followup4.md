# Stage 4 follow-up 4: the device suite, emoji tiles, then group 2

The Rust settings renderer stays **default off** and Compose stays in place. Part 1 runs the
whole instrumented suite and a fresh gallery on `aw_keyboard_ssd`, which
[follow-up 3](stage4-followup3.md#still-owed) could not, and fixes what they found. Part 2 takes
colour emoji off vello_cpu's 64 MiB glyph-atlas page. Part 3 ports settings group 2. Raw
receipts are under ignored `build/stage4-followup4/` and `/Volumes/2TB/aw-scratch/`; none
holds typed text, field text, the dictation vocabulary, learned words or clipboard contents.

## Part 1. The device suite and the gallery

### Host health

`/Volumes/2TB/aw-scratch/device-health/monitor.sh` appends one line every 5 s to
`health-20261115.log`. Each line holds:

* swap use and free memory;
* the `aw_keyboard_ssd` qemu process's RSS and CPU;
* the adb state of `emulator-5556`;
* the last adb server log line naming it.

It reads only, and never touches emulator-5554 or the adb server's lifecycle.

It ran from 09:09:57 to the end of the session, sampling every 5 s:

* **adb never dropped.** `emulator-5556` was absent until the AVD booted, offline from
  09:11:13 to 09:11:28 while it came up, then `device` in every sample after 09:11:33.
  No `adb reconnect` was needed. The adb server log's last line naming `emulator-5556` was
  still the one from shutting the AVD down on 11-14 at 23:12, so the server logged nothing
  about it all day.
* **Memory held.** Swap stayed between 3,982 and 4,006 MiB of 5,120, and free memory between
  68 and 78 percent.
* **The emulator's RSS** peaked at 8.16 GiB, during the galleries, and was 6.2 GiB at the end.

### Baseline

The AVD that follow-up 3 left in its fixture state was restored on 11-14 at 23:10.
`settings-device-baseline.py emulator-5556 .device-backup/stage4-followup3 --verify` then
passed at 09:11, before anything changed. It covered files, IME, accessibility, font scale,
rotation, animations and dark mode. Before each app run, all six streams were muted and each
one checked with `get-stream-volume`.

### Suite results

The final run of `all` on both renderers passed at 16:40 on 11-15, on a clean build of
`6d379a9c` (see [Gate](#gate)). Its receipts are in `build/stage4-followup4/suite-final8/`.
`suite-final6`, the same code built before the clean, had passed at 15:34 with the same counts.

| Mode | Result | Receipt |
| --- | --- | --- |
| ui, Compose and Rust | passed. 94 audit checkpoints each (57 of them dialogs), 12 home routes, groups 1 and 2 walked on both. Preference files are byte-identical (5,289 bytes). | `compose-ui.json`, `rust-ui.json`, `preference-parity.json` |
| group 1 walk | identical on both renderers. Rows exercised: input feedback 17, Smartbar 7, gestures 20, clipboard 24, emojis 13. None never enabled. Vibrator facts match. | `group-one.json` |
| group 2 walk | identical on both renderers. Rows exercised: Typing 15, User dictionaries 4, Physical keyboard 1, About 6, Dictation 1. On both, Typing's Languages was never enabled, because it needs this app as the system spell checker. About's Version was left unclicked, so the walk does not overwrite the clipboard. | `group-two.json` |
| text, Compose and Rust | passed. Rust: 18 InputConnection operations, 2 selection handles, handle drag, recreate mid-composition. Compose: the same edit sequence, 11 operations. The dictation vocabulary field took 9 operations on each, is multi-line on both, and was restored. Preference files are byte-identical (714 bytes). | `rust-text.json`, `compose-compose-text.json`, `vocabulary-field.json`, `text-preference-parity.json` |
| death | passed. `am kill`, new pid, bundle restored. | `process-death.json` |
| crash, light and dark | passed. Native-load failure, sanitized copy details, clipboard restored, IME fallback notice, 3 show/hide cycles, spell checker inert. | `rust-crash.json`, `rust-crash-dark.json` |
| accessibility | Rust: 0 violations, 0 known defects. Compose: 0 violations, and 258 known-defect nodes in 60 entries, all owed: Jetpref list entries with no role (209), unlabeled dialog sliders (27) and linked switches (19), FlorisBoard's card (2) and hyperlink (1) with no role; see [below](#a4-the-compose-accessibility-classification). `acceptance_complete` stays false. | `accessibility-result.json` |
| cleanup | the user's files, secure settings and font scale restored exactly; then `settings-device-baseline.py --verify` passed at 16:40. | `cleanup.json` |

The suite found these differences and walk bugs before it passed:

* **Dialog margin, and radios over the buttons** (`905068e2`). Preference parity first failed
  on the 24-entry gesture lists:
  * With a 32 dp margin, the Rust dialog ended above Compose's last visible radio (Undo), so the
    walk chose a different entry.
  * Radio circles were painted over the buttons below the list.

  Compose keeps a 16 dp margin. Every box inside a list now carries the list's clip.
* **Saved-state slots** (`ab0bcafd`). Saved state v2 merged the two scroll slots into one, so
  every editor slot moved down by one. The text stage still read the v1 slots, so it failed on
  every run (`text1`–`text3`). The slots are named now, and the version is checked first.
* **Parity receipts** (`497efa14`). A parity failure now names the differing preference keys.
  It does not name their values, which can hold field text.
* **Which nodes the walks count as rows** (`f24c4ffb`, test harness; the app was right). After
  the text fixes, the full suite failed with "Group 1 walks differ". On the gestures screen,
  Compose listed one row as never enabled and Rust listed none. Both ui stages had passed, and
  the preference files matched. The two walks differed for two reasons:
  * **A row clipped at the top was named by its summary.** Compose leaves a child that is
    scrolled out of view out of its row's merged node. A disabled row cut off by the top of
    the list was therefore labelled from its summary down, and the walk took that summary for
    a row it had not seen. The walk now leaves out a row whose top is above the list's top.
    A walk that starts at the top has already counted that row one scroll earlier, either whole
    or clipped only at the bottom.
  * **Rust's disabled rows were never counted.** Compose keeps a disabled row clickable, as a
    disabled View is; Rust does not. Neither offers ACTION_CLICK. The walk counted only
    clickable nodes, so Rust's "never enabled" list was empty by construction, and the
    comparison could not catch a row that stayed disabled on Rust alone. In a diagnostic run,
    Compose saw disabled rows 107 times across gestures, clipboard and input feedback, and Rust
    saw none.

  `SettingsHarness.rows` now picks a walk's rows for both groups. It takes labelled nodes that
  are clickable or disabled, and leaves out fields, sliders, the screen's chrome, nodes inside
  another row, and rows clipped at the top. Group 2 used to name rows from a title map; it now
  uses the same helper. The assertion is unchanged.
* **Smartbar's fixed row** (`6d379a9c`, app). Once disabled rows counted on both renderers,
  the next run (`suite-final5`) failed the same assertion on the Smartbar screen. Rust listed
  "Auto-expand/collapse" as never enabled; Compose did not list it at all. The row is a
  Jetpref `Preference` with no `onClick`, `enabledIf = { false }`, and a trailing
  `Switch(checked = true, onCheckedChange = null, enabled = false)`:
  * without `onClick`, Jetpref adds no clickable modifier, and the condition only sets the
    ListItem's alpha to 0.38;
  * Material's `ListItem` merges its texts into one node with no role and no disabled state;
  * a Switch with no `onCheckedChange` adds no semantics.

  So TalkBack reads the row as enabled text, and Rust exposed a disabled button. Group 1's
  frozen reference had pinned "button" for it, because it predates the `text` role group 2
  gave Jetpref's passive rows. The row now takes that role, and `Control::exposed_enabled`
  reports a passive row enabled, as Compose does. It is still drawn faded and checked, so the
  pixels do not change. `SettingsGroupOneMatchTest` checks the role and the reported state, and
  the mutations `fixed-row-button` and `fixed-row-reported-disabled` each fail it. Before the
  walk fix, neither walk counted the row, so the two agreed by accident.

### Jetpref's store stops writing after a cancelled write

Re-running the text stage for group 2 showed two problems:

* The preference files differed by one byte (714 and 713) on
  `smartbar__shared_actions_expand_with_animation`, a key the IME writes, not the screen. The
  Compose test had raised the IME and the Rust test had not. The Rust test now taps the field
  through accessibility, as the Compose path does.
* About one Rust run in five then failed with "vocabulary not saved at step 3" (or 6). Compose
  never failed this way in about 15 runs.

Diagnostic logs and an in-process thread dump traced both to Jetpref's DataStore (the pinned
snapshot 20251119T222500Z):

1. `PreferenceData.set` updates the cached value and the Flow. It then queues a persist event
   and waits for that event's answer on its `done` channel, which it consumes.
2. The store's one loop handles each event inside `runCatching`, then sends the result on
   `done`.
3. If the caller is cancelled while it waits, consuming `done` cancels it. The loop's `send`
   then throws CancellationException outside the `runCatching`, and the loop ends without a word.
4. From then on, every write in the process updates the cache and never reaches the file. Each
   caller fails or waits for an answer that never comes.

The IME cancels its own writes. NlpManager collects `NlpState.expanded` with `collectLatest`
and sets two Smartbar preferences inside it, so a quick change of state cancels a write in
flight. Leaving a screen while it writes does the same.

* On Compose, screens keep reading the cache, so nothing looks wrong. The file just goes stale,
  which can leave it a byte off the other renderer.
* On Rust, SettingsPlatform's one write queue waits on the dead store, so every later write
  stalls. That was the flake.

**`SettingsPreference`** (`e5de55fd`) wraps every preference the model declares. A write
that has started runs to its end in `NonCancellable`, and the caller sees its own cancellation
afterwards.

The wrappers are typed as `PreferenceData`, because Jetpref's KSP processor declares only
properties of that type. Typed as the wrapper, 48 enum and custom preferences dropped out of
the model: the store would neither load nor save them. `SettingsUiStoredFileMatchTest` caught
that.

`SettingsUiWriteCancellationMatchTest` holds one write in a test writer, cancels its caller,
then writes a second preference:

* On the raw reference model, the second write fails. That shows the defect is real in the
  Jetpref we ship. If an update fixes it, this half fails and the wrapper can go.
* On the production model, the cancelled caller waits for its write, and the second write
  reaches the file.

It runs this for both `set` and `reset`. It also checks that the model declares the same keys
as the reference, and that every one is wrapped. Store calls are bounded, so a write that
never finishes fails the test instead of hanging it. The mutations `write-cancellable`,
`reset-cancellable` and `custom-unwrapped` each fail it.

### Three more causes of text-stage flakes

With the store fixed, the text stage still failed now and then. Three separate causes turned up:

1. **The IME's start finished the composition** (test harness). FlorisBoard's start plan, which
   runs inside `onStartInputView`, includes `finishComposingText`. Edits sent before the IME
   started could have their composition finished halfway through. Step 3's `setComposingText`
   then appended instead of replacing, giving 10 units where 8 were expected. No typing user
   can do this, because the keyboard is up before the first key.
   * `SettingsHarness.awaitIme` now waits until the in-process `FlorisImeService` shows its input
     view for a multi-line editor, then for an idle main looper. The start plan runs on the main
     thread, and its connection calls arrive on the main looper.
   * Both editor paths wait before the vocabulary sequence starts (`885756d0`).
   * A failure receipt records only three facts: whether the service was found, whether its
     view is shown, and whether the field is multi-line. It records no text.
   * Along the way, settling on accessibility nodes did not work: the IME's window belongs to
     this package too, so `settle` read the keyboard and never settled.
2. **The Rust editor's keyboard request could be lost** (product, fixed). The `editor` effect
   asked for the keyboard only through `WindowInsetsController.show(ime())`.
   * When no IME was attached yet, as after the instrumentation force-stops the package,
     the system logged `onFailed at PHASE_SERVER_UPDATE_CLIENT_VISIBILITY` and did not retry.
     No IME came for the rest of the run.
   * Compose's text field goes through androidx's `SoftwareKeyboardControllerCompat`, which
     also calls `InputMethodManager.showSoftInput`. The system log shows that pair for every
     Compose show.
   * The Rust adapter now uses the same controller (`20353a1d`).
3. **Restoration wrote under a live process** (test harness). `restore_files` bound the IME
   first and wrote the files second. The bind started the app process, which loaded the old
   bytes into memory. Jetpref writes the whole file on any write, so that process's next write
   put the old values back. Cleanup's re-checks caught this twice, both times after a failed
   run, as "Exact original file restoration failed". It also meant a cleanup could leave the
   user's keyboard running on test values.
   * The files are now written while the process is dead: force-stop, write, confirm no pid,
     up to five tries.
   * Then the original IME is bound again, so its process loads the restored bytes, and the
     hashes have to hold steady for 1.5 s before they are verified (`88de4a53`).
   * Eight cleanups with the IME alive on modified preferences passed under both orders
     (`build/stage4-followup4/restore-stress/`), so this stress did not reproduce the race. The
     order is fixed by construction, not shown by a reproduction. A cleanup now takes 3.2 s
     instead of 1.5 s.

Receipts are under `build/stage4-followup4/text-loop*/`. Each loop ran the text stage, both
renderers, 10 to 12 times:

| Loop | Code | Result |
| --- | --- | --- |
| `text-loop` | store fixed | 6 of 10 passed. 4 Rust runs failed at step 3 (cause 1). |
| `text-loop2`–`5` | detecting the IME | Failed attempts to detect the IME: insets, UiAutomation windows, then node settling. |
| `text-loop6`, runs 1–8 | `awaitIme`, old restore order | 8 of 8 passed. |
| `text-loop6`, runs 9–12 | plus the new restore order | 2 of 4 Rust runs had no IME attached (cause 2). |
| `text-loop7` | all three fixes | **12 of 12 passed.** Preference files byte-identical (714 bytes) every run, 9 field operations per renderer, the user's files restored. |

### A4: the Compose accessibility classification

The Rust path had no violations and no known defects. The Compose path had two kinds of
problem, both inside Jetpref, which this repository uses as a pinned binary snapshot.

* **46 unlabeled nodes**, named and reported as owed, as follow-up 3 set up (27 entries by
  control):
  * `jetpref-linked-switch-unlabeled`: the bare switch that ListPreference draws beside a
    linked row;
  * `jetpref-dialog-slider-unlabeled`: the Material Slider in DialogSliderPreference's dialog.
* **206 nodes with "missing semantic role"**, which the classification did not cover, so
  `accessibility_passed` was false. Every one is an entry in a ListPreference dialog: the
  gesture actions, the clipboard sync and sort modes, the keyboard screen's lists, and so on.
  Jetpref d6e12dda's `ListPreference.kt` builds each entry as a `Row` with
  `Modifier.selectable(...)` and no `role = Role.RadioButton`. The `RadioButton` inside it has
  `onClick = null`, so it carries no semantics either. TalkBack reads the entry's label and
  selected state, but does not say it is a radio button. Rust gives these entries the radio
  role.

The harness now names the second shape too: `jetpref-list-entry-no-role`. It applies only on
the Compose path, only to a node whose sole violation is the missing role, and only while a
dialog is open. Like the other two, it is reported per dialog as owed, and
`acceptance_complete` stays false until Jetpref is forked or fixed upstream.

### Gallery and strict diff

`scripts/settings-pictures.py` captured every group 1 screen (top and lower pages) and all
of its dialogs on both renderers, in light, dark and font scale 2.0: 66 pairs per theme.
`scripts/settings-picture-diff.py` compared them strictly: 3/255 per channel, a one-pixel
neighbour band, the 72 px system bars excluded, no alignment.

The first capture found five differences. Each is fixed, pinned by a Rust test, and has a
mutation that fails that test:

* **Scroll-forward at the end** (`9a09dd1d`). A fling can stop a fraction of a pixel short of
  the maximum. The screen shows the rounded end, but the accessibility node still offered
  ACTION_SCROLL_FORWARD, so the gallery's second scroll on the Rust clipboard screen moved
  nothing. Compose's ScrollState answers from its whole-pixel value; Rust now does too.
* **Stretch overscroll** (`fb97b200`). Rust's clipboard, gestures and input-feedback screens
  rested 13 px short of the end after the second scroll. An injected drag past the bottom
  lifts with a small backward velocity, and Rust flung the content back. Compose's
  `verticalScroll` stretches instead: the edge takes the pixels the scroll could not, a
  reverse drag or fling relaxes that first, and a fling toward a stretched edge is absorbed.
  `overscroll.rs` models where that leaves content. The drag reference in
  `SettingsDragMatchTest` follows the same rules now, because its reversal gesture runs into
  the end.
* **One-line switch rows** (`4a54d6c1`) are 64 dp, not 56: ListItem puts 8 dp around a 48 dp
  switch target. Every row below a one-line toggle on the clipboard screen sat 8 dp high.
* **Disabled switches** (`d752cb96`) use Material 3's disabled tokens, under the row's 0.38
  fade, instead of the enabled colours faded. This is what the **Smartbar's fixed row**
  (checked and disabled) showed against its Compose picture; the clipboard screen's disabled
  toggles showed it too.
* **Radio text width** (`e4db0cd1`). A dialog entry's text box is the row less the radio and
  its gap, which Compose measures in whole pixels. Rust used fractional widths, so the
  enable-audio-feedback dialog's labels wrapped differently.

The second capture (`/Volumes/2TB/aw-scratch/followup4/run2-ld`) matched 65 of 66 light pairs
and 60 of 66 dark pairs. The light failure was the radio text above. The five dark failures
were all dialogs over a screen with a switch: a thumb behind the scrim sat a pixel off (max
error 4). Rust placed switches in fractional pixels, where Compose lays them out whole:

* the track is centred in the 48 dp target with `Alignment.Center`, which rounds;
* the 52 dp track overflows the 48 dp target, offset by an integer division;
* M3's ThumbNode sizes the thumb and offsets it along the track with `toPx().toInt()`;
* a linked switch's divider is a 2 px line centred on its box's left edge.

**Whole-pixel switches** (`94db57aa`) does it Compose's way. At 2.625 the track is
137 by 84 px, 907 px from the left on a 1080 px screen. A checked thumb is 63 px, 63 px along
and 11 px down. `switches_sit_on_the_whole_pixels_compose_lays_them_out` checks every switch
on the keyboard screen. The mutations `switch-half-pixel-track` and `linked-divider-inside`
each fail it.

The final capture (`build/stage4-followup4/pictures-final`, 15:35 to 16:29) covered the
keyboard screen, group 1 and group 2, every screen and dialog, on both renderers: 76 pairs per
theme. **Light matched 76 of 76 pairs strictly, and dark 76 of 76.** The largest channel error
outside the edge band was 2/255. That includes the Smartbar screen, whose fixed row changed role
but not pixels. Both renderers skipped the same two rows: Smartbar's Action row placement, which
shows only for the extended layout, and Typing's Languages, which is enabled only while this
app is the system spell checker.

That capture left its agent-device session open, and the session's helper held UiAutomation,
so the next instrumented run could not start. `settings-pictures.py` now closes the session
when it finishes (`2e66c5d7`, test harness). The [Gate](#gate) section has the
details.

**Font scale 2.0 matches on none of its 76 pairs.** Most of that is the allowed difference
[follow-up 3](stage4-followup3.md#a2-fresh-pictures-and-the-strict-diff) recorded: Rust wraps
titles, summaries, the app bar title and dialog titles where Compose ellipsizes them, so
everything below the first wrapped row moves down. Two more differences remain, both owed:

* **Glyph ink and dialog buttons.** Inside the dialogs, where nothing wraps, every line sits
  at the same height and spans the same pixels on both renderers. Glyph edges still differ:
  measured glyph by glyph in the reset dialog, Rust's ink sits 0.04–0.23 px (typically
  0.12 px) left of Compose's. The offset is the same along a line, so the advances agree.
  The dialog buttons' labels also sit 1–3 px right of Compose's. At font scale 1.0 the same
  text and buttons match to 1/255. So these come from the larger, non-linearly scaled sizes
  (68.25 to 94.5 px). The cause was not found.
* **The preview field clips its text.** PreviewKeyboardField is a single-line `TextField`
  fixed at 56 dp. At font scale 2.0 its placeholder is taller than the 24 dp left inside the
  16 dp padding, so Compose clips it: the descenders are cut off. Rust centres the whole
  line. This is the same kind of difference as the ellipsizing, so it stays until the user
  decides between legibility and parity.

## Part 2. Colour emoji without the atlas page

Commit `522eb17a`, measured by `c36db121`.

vello_cpu 0.4's glyph atlas allocates a fixed 4096×4096 RGBA page (64 MiB, filled when it is
created) for the first colour glyph it draws. The CPU backend now keeps one trimmed tile for
each colour glyph, size, variation and paint. Each tile is drawn as an image on whole pixels,
which matches the atlas's nearest sampling. The tiles share a 4 MiB budget that never drops
anything the last frame drew. Scaled or image-painted text, and glyphs too big for the scratch
canvas, are drawn directly. The Hybrid backend is unchanged.

Tiles match direct drawing to within 3/255 for COLR and exactly for CBDT, on light and dark
backgrounds. `color_glyph_memory.rs` fails if drawing emoji grows the process by 16 MiB or
touches the atlas. It measured +67 MiB with the atlas and +1.2 MiB now.

On the device, `scripts/keyboard-memory` measured the IME process at five points, all in the
same process: settings open, keyboard shown, after typing, inline colour-emoji candidates, and
the emoji panel opened and swiped. Receipts: `build/stage4-followup4/memory-before` (`c36db121`)
and `memory-after` (`522eb17a`).

| Point | PSS KiB before | PSS KiB after | Native heap KiB before → after | Draw p50 / p95 ms before → after |
| --- | ---: | ---: | ---: | ---: |
| settings | 101,161 | 101,212 | 16,482 → 16,478 | no frames |
| keyboard shown | 144,623 | 140,886 | 57,667 → 57,519 | 8.03 / 15.23 → 8.97 / 17.30 (2 frames) |
| typed | 144,877 | 144,277 | 57,919 → 57,803 | 0.55 / 0.72 → 0.54 / 0.80 |
| emoji candidates | 209,977 | **153,599** | 123,975 → 58,579 | 0.51 / 1.17 → 0.50 / 0.69 |
| emoji panel | 223,097 | **157,569** | 124,659 → 58,991 | 0.74 / 1.40 → 0.77 / 1.01 |

The emoji points drop by 55–64 MiB, which is the atlas page. Draw times for colour emoji are
unchanged or better. "Keyboard shown" has only two frames, the first of which includes setup,
so its percentiles are not a trend.

## Part 3. Settings group 2

### Ported

These routes are now native (`catalogue::native_routes`), still behind the default-off
selector:

| Route | Screen | What it needed |
| --- | --- | --- |
| `settings/typing` | TypingScreen | ForgetLearnedWordsDialog, the auto-space feedback card, SpellCheckerServiceSelector |
| `settings/dictionary` | DictionaryScreen | rows that are enabled by a switch and lead to Compose screens |
| `settings/other/physical-keyboard` | PhysicalKeyboardScreen | the attached-keyboard fact, ACTION_HARD_KEYBOARD_SETTINGS |
| `settings/about` | AboutScreen | the app-icon header, copying the version, URLs, the licence routes |
| `settings/dictation` | DictationScreen | ModelStatus, the note, the vocabulary OutlinedTextField |

New pieces. Rust decides and draws each one; where a platform effect is needed, it returns a
plan for Kotlin to execute.

* **A confirmation with its own message** (`Kind::Confirm(&Ask)`). It has a title, a
  message, Cancel and Delete, and no neutral button. Confirming returns
  `["forgetLearnedWords", <toast>]`. Kotlin erases the learned words through `NlpManager` and
  shows the toast Rust worded. The words never reach Rust, a log or a receipt.
* **An info card with a hyperlink** (`Kind::Card`). It shows only while auto-space is on. The
  link is its own control (6000 + row) with the link role, and it returns `["url", …]`.
* **The spell checker cards** (`Kind::Spell`), read from the facts in Settings.Secure
  (`spell_checker_enabled`, `selected_spell_checker`):
  * the error card, with FlorisErrorCard's fixed red and white;
  * the warning card, with FlorisWarningCard's yellow and black;
  * the selected service's card, showing its label (or "Unknown"), its package and its icon.

  Kotlin parses the package exactly as `ComponentName.unflattenFromString` does, and the JVM
  tests run that same parse. Languages is enabled only while FlorisBoard is the system spell
  checker. Kotlin observes the two settings and pushes new facts when they change.
* **An app-icon header** (`Kind::Header`): AboutScreen's 64 dp icon, centred 24 dp below the
  app bar, with the app name in 24 sp SemiBold. Pictures reach Rust through a new
  `settingsImage` JNI entry as ARGB at the drawable's intrinsic size. The image node carries a
  content description and no text, as Compose's `Image` does.
* **Copy to clipboard with a toast**. The version row returns `["copy", "<name> (<code>)",
  <toast>, <error toast>]`. The copied text is the build's version, never anything the user
  typed.
* **External URLs and intents**:
  * `url` goes through `launchUrl`; the changelog URL fills in `{version}`;
  * `intent` is either `hardKeyboardSettings` or `spellCheckerSettings`.
* **ModelStatus**. Opening the screen returns `checkModels`. Kotlin then reports the quick
  state at once, and the SHA-256 check from a background thread, as facts. Rust words each
  state, including the damaged files' names.
* **The dictation model field**: DictationScreen's vocabulary `OutlinedTextField`, built on
  the same Rust editor as the preview field.
  * One editor owns the InputConnection at a time. Tapping the field (or `focus` with a
    control id) parks the preview field's editor and returns `restartInput`.
  * Every change is written to `dictation__vocabulary`, as the screen's `onValueChange` does.
  * Enter inserts a new line.
  * The endpoint reports Compose's multi-line EditorInfo: `TYPE_TEXT_FLAG_MULTI_LINE`,
    `IME_FLAG_NO_ENTER_ACTION` and `IME_FLAG_NO_FULLSCREEN`.
  * Saved state keeps only the preview field's editor; the vocabulary comes back from its
    preference.
  * The field's minimum height is 120 − 16 dp, because `heightIn(min = 120.dp)` comes before
    `padding(16.dp, 8.dp)`. It has 4 lines, a 1 dp outline, and 2 dp in primary when focused.

Strings come from Android resources through the ID map, which now has 372 entries. Five
literals in the Compose sources became resources so both renderers read the same text: the
auto-space card's text, its link label and its URL, "Unknown", and the app icon's content
description. Preference writes go through the Jetpref serializers as before.

### Deferred, with reasons

* **Other** (`settings/other`), and with it **ColorPickerPreference**. Other's accent colour
  uses `ColorPickerPreference(enableAdvancedLayout = true)`. Its neutral button switches the
  dialog between Presets (a `LazyVerticalGrid` of swatches) and Custom, which is Jetpref's
  `JetPrefColorPicker` (d6e12dda, 710 lines). Custom needs four widget kinds the Rust settings
  UI does not have:
  * a saturation/value plane with a two-dimensional drag;
  * a hue bar;
  * slider tracks drawn as gradients;
  * hex text fields that stay in sync with the plane.

  That is more than this group's pieces, so it goes to a later group, which should port the
  picker as a whole. Physical keyboard is still reachable while Other stays in Compose,
  because Compose hands `settings/other/physical-keyboard` back to Rust.
* **Theme** (`settings/theme`). Besides the same colour picker, it needs:
  * two `LocalTimePickerPreference`s, each a Material time picker dialog with a clock dial;
  * the day and night theme labels, read from the extension index (`indexedThemeConfigs`, a
    Flow);
  * `AddonManagementReferenceBox`.

  None of these are in this group.

### Proof

Each new piece is checked on the JVM against a frozen Compose reference, by Rust tests, by
mutations that must fail them, and on the device.

* **The Compose reference.** `ReferenceGroupTwoScreens.kt` is a test-only Kotlin copy of the
  five screens' decisions. `SettingsGroupTwoMatchTest` holds Rust against it:
  * every combination of the conditions each screen reads: its preferences, the spell checker
    facts, the attached keyboard, and both model states, checked or not;
  * every row's visibility, enabled state, role, text and summary;
  * the card's link and the header's image as nodes of their own;
  * whether the preview field shows;
  * seeded random action runs that click rows, confirm or cancel, follow links and change
    facts, comparing each plan with the original `onClick`.

  The fact parser is the one Kotlin runs. The test uses the real Roboto, because a link the
  width of no text is no node at all.
* **The editor.** `SettingsEditorMatchTest` drives the vocabulary field with Compose's own
  `EditProcessor` from its saved text. It checks that every change is written as it is, that
  Enter is a new line, and that the preview field gets its own editor back.
* **Rust tests.** `settings_ui_group_two.rs` has ten tests: the native routes, the spell card,
  the confirmation's own buttons, the auto-space card and its link, About's copy, URL and
  header, Physical keyboard, ModelStatus, the vocabulary's editor and its minimum height, and
  the cards' colour, tracking and icon box. `android_text.rs` adds FreeType's advance rounding.
* **Mutations.** `scripts/settings-ui-mutations.py` has 39 cases (now in `scripts/settings_ui_mutation_cases/general.py`): sixteen for group 2 (four
  of them the [gallery fixes](#the-group-2-gallery)), the seven part 1 fixes, two for
  [Smartbar's fixed row](#suite-results), three for the
  [store fix](#jetprefs-store-stops-writing-after-a-cancelled-write), and the earlier ones.
  The final run (`build/stage4-followup4/mutations-final/result.json`, 16:05) caught all 39:
  26 with fresh JVM assertion failures and 13 with Rust test panics. With the sources
  restored, the JVM tests and every affected Rust test binary pass, and git shows no source
  change.
* **On the device.** The final suite (`suite-final8`) walked all five screens on both
  renderers, and the walks matched row for row (see [Suite results](#suite-results)). On
  Typing, the walk exercised 8 toggles, the incognito dialog, the forget-learned-words
  confirmation, the spell checker card's intent, the User dictionaries route, the auto-space
  card shown and hidden, and its hyperlink's URL. It also exercised User dictionaries' two
  switches and two routes, Physical keyboard's switch, About's three URLs and two licence
  routes, and Dictation's switch. The preference files came out byte-identical. The
  vocabulary field took the same 9 operations on each renderer and was restored. The
  [group 2 gallery](#the-group-2-gallery) and the [final gallery](#gallery-and-strict-diff)
  hold the pixels.

### The group 2 gallery

`settings-pictures.py --sections group-two` captures each group 2 screen from its top, its
lower pages, and its dialogs: 10 pairs per theme. It clears the dictation vocabulary first, so
no picture holds what the user wrote. Languages is skipped unless this app is the system
spell checker. Compose marks the disabled row's text node as enabled, so the gallery reads the
`selected_spell_checker` fact instead of the node.

The first full light capture matched 8 of 10 pairs. The two Typing pages below the top
differed on the auto-space card and the spell checker card. Commit `0014e222` fixes both.
Each fix has a Rust test and a mutation that fails it.

* **Card text colour.** An M3 `Card`'s content colour is
  `contentColorFor(surfaceContainerHighest)`. materialkolor's scheme makes surfaceVariant equal
  to surfaceContainerHighest, so that resolves to onSurfaceVariant (71, 70, 72 in light), not
  onSurface. Kotlin passes it as palette slot 13, config field 55. Mutation:
  `card-content-onsurface`.
* **titleMedium tracking.** The resolved Material library's `TypeScaleTokens` has 0.2 sp,
  not the published 0.15 sp. Mutation: `title-medium-published-tracking`.
* **Advances of an unhinted font.** The device's Roboto is variable and carries no hinting
  program, so FreeType gives Minikin its linear advance, rounded in three steps:
  1. the HVAR delta in whole font units;
  2. the scaled advance as 26.6 fixed point;
  3. `FT_PIX_ROUND`.

  Roboto's `e` at weight 500 and 42 px is 22.497 px. That is 1440/64 = 22.5 in 26.6, and so
  23 px, where rounding the linear value gave 22. A throwaway `StaticLayout` probe on the
  device gave each glyph's position, and Rust now reproduces all of them. Mutation:
  `advance-linear-round`. The whole-unit HVAR step has no unit test, because no variable text
  font is committed. The gallery is its proof.
* **The error card's icon box.** FlorisErrorCard pads a 24 dp icon by 12 dp all round. Compose
  rounds each to whole pixels, so the box is 32 + 63 + 32 = 127 px at 2.625, not 48 dp's
  126. Mutation: `card-icon-box-48dp`.

After the fixes the group 2 light capture (`build/stage4-followup4/pictures-g2`) matched 10 of
10 pairs strictly. The [final gallery](#gallery-and-strict-diff) covers every theme.

## Gate

`./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug`
passed at 15:43 on 11-15, on `6d379a9c`, and again at 16:46 on the final tree, with the same
results:

* 846 JVM tests in 211 result files across every module, with no failures, errors or skips;
* `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`, with
  and without the Latin oracle, and the native declarations check;
* no Kotlin or Java compiler warnings;
* lint reports only the allowed IDs: AndroidGradlePluginVersion (1), NewerVersionAvailable (3)
  and ChromeOsAbiSupport (1). (An earlier count of 2 for the last one also read
  `lib/latinime`'s report from 11-07, which the gate does not write.)

No file touched here has more than 500 lines of code, and no `#[allow]` or `unsafe` was added.

The first final run failed one test, `ComposeStabilityLinkageTest`. `AppPrefs.kt` had been
recompiled on its own at 14:17 for `e5de55fd`, and `FlorisPreferenceModel$Clipboard` came out
without its `$stable` field. Two callers compiled on 11-14, `RendererClipboardKt` and
`ClipboardInputLayoutKt`, still read that field. This is the stale incremental Compose output
that [stage 2](stage2-settings.md) and [stage 3B](stage3b-package8.md) recorded. `:app:clean`,
then a compile with the build cache off, compiled callers and owners together, and the class
has the field again. The test is unchanged and passes. The failed log is kept as
`/Volumes/2TB/aw-scratch/followup4/gate-before-clean.log`.

The fixture APK that `suite-final6` and the final gallery ran was built before the clean, so it
carried those two stale callers. Neither is on a path the suite or the gallery reaches: they
belong to the keyboard's clipboard panel, and a stale caller fails with `NoSuchFieldError` on
first composition rather than drawing differently. The suite was run again on a clean fixture
build (`suite-final8`) and passed with the same counts.

That rerun first failed to start: "UiAutomationService ... already registered". The gallery
drives the device through an agent-device session and left it open, and its helper still held
UiAutomation. Closing the session let the suite run. `settings-pictures.py` now closes its
session when it finishes (`2e66c5d7`). A small capture run afterwards left no helper
behind.

## Still owed

* **Font scale 2.0 pixels** ([above](#gallery-and-strict-diff)). Two things are left:
  * glyph ink about 0.12 px apart, and dialog buttons 1–3 px apart, at the non-linearly
    scaled sizes;
  * the user's call on legibility against parity. Rust wraps where Compose ellipsizes, and
    draws whole the preview-field placeholder that Compose clips.
* **Jetpref accessibility** ([A4](#a4-the-compose-accessibility-classification)). On the
  Compose path, list entries have no radio role, and the linked switch and the dialog slider
  have no labels. `acceptance_complete` stays false until Jetpref is forked or fixed upstream.
* **The switch in a dialog title.** JetPrefAlertDialog's `trailingIconTitle` holds a
  `Switch(padding(start = 16.dp))`. Its whole-pixel geometry was not derived, so it keeps the
  fractional placement it had. The input-feedback dialogs show it, and the haptic one matched
  strictly in both themes, so the placements agree at 2.625. Other densities are unchecked.
* **Other, ColorPickerPreference and Theme**: deferred, [with reasons](#deferred-with-reasons).
* **A real TalkBack session on a phone.** The suite checks nodes, roles, labels, states and
  actions through UiAutomation on the emulator. `real_talkback_phone_session` stays `owed` in
  the accessibility receipt.
* **Jetpref's store defect upstream.** `SettingsPreference` works around it here; the fix
  belongs in Jetpref's `DataStore` (send `done` inside the loop's `runCatching`, or never
  let a caller's cancellation reach the channel). `SettingsUiWriteCancellationMatchTest` says
  when an update has fixed it.
* **Writes queued when a Rust screen is destroyed.** SettingsPlatform drains its one write
  queue in the activity's `lifecycleScope`. A write that has started finishes (see the store
  fix), but plans still queued behind it are dropped when the activity is destroyed. Compose
  starts each write at once, so it has no queue to lose. Draining the queue in a process
  scope would close this; nothing in the suite hit it.
* **Roleless Compose controls on Typing.** FlorisSimpleCard's `Card(onClick)` and
  FlorisHyperlinkText's clickable `Text` carry no role. The walk names them
  `floris-card-no-role` and `floris-hyperlink-no-role` and reports them as owed. Rust gives
  both nodes the button role (`53b7c640`).
* **Disabled rows.** On Compose a disabled row's node is clickable, as a disabled View is; on
  Rust it is not. Neither offers ACTION_CLICK, so TalkBack behaves the same, and the walk
  counts disabled nodes as rows on both.
* **Screens outside the port.** The system and internal user dictionaries and both licence
  screens stay in Compose on both renderers. The walk opens them, and on Rust it checks they
  open in `ComposeSettingsFallbackActivity`, but it does not audit them.
* **The whole-unit HVAR step** of FreeType's advance rounding has no unit test, because no
  variable text font is committed. The group 2 gallery is its proof.
