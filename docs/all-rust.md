# Everything in Rust

2026-11-21. The goal is the whole app in Rust: every decision, piece of state and drawing. What
stays on the JVM is only what Android forces, as a thin layer of adapters, and as much of that
layer as can be generated is generated. This page is the path there. It came from a council
review by Claude Fable 5.2 and GPT-6.2 Astra (scores 7/10 and 8/10 for the direction as it
stood), checked against the code and decided by the owner ([decision](decisions.md#everything-moves-to-rust-2026-11-21)).

[compose-removal.md](compose-removal.md) stays the order for deleting Compose screens; it is
phase 4 below and the keyboard step K is phase 1. This page adds what that one does not plan:
moving the keyboard's state out of Kotlin, the Kotlin–Rust boundary, and the reference code.

## Where it stands (0b8b5320a)

* **Production Kotlin:** about 52,000 non-blank lines in 494 files. **Rust:** about 87,800.
  **Java:** none in production; 1,490 lines in tests and tools.
* **LatinIME is ported.** The app ships no C++: its native libraries are `libawnative` (Rust),
  sherpa-onnx with onnxruntime (speech), and one androidx library. `lib/latinime` (14,868
  lines) is built only on the host, as the oracle `latin-oracle` compares Rust against. The
  `stage6/*` branches were fully merged and are deleted. (Phase 6 has since turned the oracle
  into golden files and deleted both: [record](phase6-oracle-goldens.md).)
* **Rust draws; Kotlin still decides much of what.** `RustKeyboardView` builds its snapshot from
  Kotlin's `TextKeyboard` and `TextKey`, and Kotlin drives Rust through 381 `external fun`s.
  The state owners are still Kotlin: `KeyboardManager`, `NlpManager`, `EditorInstance`,
  `ThemeManager`, `ClipboardManager`, and `ime/text`, `ime/keyboard`, `ime/nlp`, `ime/editor`,
  about 10,000 lines together.
* **Compose:** the keyboard fallback (about 5,300 lines), settings steps 5–12 (about 9,700),
  and the shared `lib/compose`, `lib/color` and Compose half of `lib/snygg`.

## The end state

Two Rust sessions own everything: `ImeSession` for the keyboard and `AppSession` for settings.
Kotlin sends facts (lifecycle, `EditorInfo`, packed `MotionEvent`s, insets, configuration,
results of asynchronous work) and carries out the plans Rust returns (InputConnection
operations, haptics, intents, file-picker requests, accessibility events). The boundary falls
from 381 functions to a few dozen typed channels.

**Hand-written Kotlin that stays, about 3,000 lines** (estimates 2,500–6,000 until the mixed
files are sorted):

| Piece | Lines |
|---|---:|
| IME service, plain-View root, surface, touch, insets | 600 |
| InputConnection executor, inline autofill, extract view | 350 |
| `AccessibilityNodeProvider` transport for the keyboard and settings | 350 |
| Settings Activity host: text endpoint, Back, file picker, permissions | 400 |
| Read-aloud AccessibilityService, overlay, TTS, AudioTrack, audio focus | 450 |
| Clipboard listener, media provider, share target | 300 |
| Strings, haptics, network, `ImageDecoder`, files | 350 |
| Microphone | 150 |
| Crash screen that works when the native library fails to load | 150 |
| Spell-checker service | 80 |
| Library loading and handle cleanup | 80 |

These entry points are written by hand. Generating an `InputMethodService` from Rust would only
move its upkeep into a generator and hide Android's deprecations from lint.

**Generated, about 2,000–3,000 lines:** the transport between them and Rust (below).

**Not worth moving to Rust:** GIPHY's HTTP and TLS (megabytes of binary to replace about 75
lines), audio playback (the Mac port already kept realtime audio out of Rust; focus,
permission and system TTS stay JVM regardless), ICU, `ImageDecoder`, and a `NativeActivity`
for settings (it loses InputConnection and accessibility).

## The boundary: generated from Rust

* **Gradle keeps driving cargo** (`cargoBuildAndroid`, `cargoBuildHost` in
  `lib/rust/build.gradle.kts`), and the JNI layer keeps `jni-sys`.
* **One in-repo host crate generates the boundary** from typed Rust declarations: the Kotlin
  `external` declarations, operation and field constants, packet readers and writers, and the
  Rust `extern "system"` stubs. A Gradle task runs it before Kotlin compiles and writes into
  `build/`; a `--check` mode runs in `:lib:rust:check`. It replaced the `GenerateRenderProtocol`
  task and its regex: `RenderProtocol` and `InputProtocol` now come from `crates/wire`.
* **`checkNativeDeclarations` learns generated sources and JVM descriptors.** Today it finds
  Kotlin through git and compares only export names, so a `Long` that became an `Int` passes.
* **Not UniFFI:** its Kotlin goes through JNA, slower than the 0.04–0.1 µs JNI crossings
  measured here; it passes strings as UTF-8, which breaks the rule that text crosses as UTF-16
  with lone surrogates kept ([rust.md](rust.md)); and its generated code carries `@Suppress` and
  `#[allow]`. **Not jni-rs:** it eases hand-written JNI but generates nothing.

## Phases

Phases 0 and 2 start now. Phase 1 waits for the owner's phone session. Phases 3 and 4 can run
side by side once 2 lands, so the operations they add are generated rather than written by
hand.

### 0. Truth and a ratchet (S, done)

* A script regenerates the [stage 5 inventory](stage5-inventory.md) ledger at HEAD: every
  production Kotlin and Java file, its size, and its class (stays JVM, generated, port, delete)
  from a checked-in table. A production file with no class fails it.
* A gate check that hand-written JVM files never grow: a budget per file, a new file needs an
  entry, and the budget only tightens on its own. Raising one is a visible edit with a reason.
  (Grade item A7 found `NlpManager` growing from 363 to 428 lines.)
* One ledger of device checks still owed, gathered from the stage documents:
  [owed-checks.md](owed-checks.md). The live JVM ledger is [jvm-ledger.tsv](jvm-ledger.tsv).
* **Exit:** the gate runs both checks, and growing one file on purpose fails it.

### 1. Phone checks, then the Compose keyboard (S–M, waiting on the owner)

* Every owed check, with its device and doc, is in [owed-checks.md](owed-checks.md). The
  phone entries for this phase are P1–P5 there.
* The four checks in [compose-removal.md](compose-removal.md#k-the-compose-keyboard): real
  TalkBack, the Samsung font, font scale 2.0, and draw p95 at most 2 ms on the S24 Ultra at
  120 Hz (`cpu-trials.py` twice, tap positions remeasured for 1440x3120, plus a Perfetto
  trace). Add the quick hide-and-show by hand.
* Repeat the package 8 deletion dry run, then delete the keyboard scope and the
  `keyboard__rust_renderer_prototype` switch.
* **Exit:** results recorded, full gate, Compose keyboard gone.

### 2. The generated boundary (M)

* **Pilot on `ShiftNative`** (three functions: primitives and a string). Keep its ABI; host and
  emulator matches pass; a deliberate symbol or descriptor mutation fails the check.
  **Done (2026-11-21):** generated by `crates/boundary`, ABI unchanged; the check now compares
  all 381 native methods' JNI types. On the emulator (`aw_keyboard_ssd`), one shift typed "Hi",
  a double-tapped shift locked caps for "HI", and two quick spaces typed ". ".
* **Then the operation constants** (grade item A6: about 100 literal op calls in 30 files):
  declare them in `settings`, `utilities`, `backup`, `editor` and `nlp`, generate the Kotlin
  side, make the raw `call` internal.
  **Done (2026-11-21):** declared once in `crates/wire` for eleven families of entry points,
  Kotlin generated into `build/` without a regex, and a clash or one-sided rename fails the
  build ([rust.md](rust.md#operation-codes-and-packets-crateswire)). On the emulator, "teh"
  composed with "the" offered and a space corrected it, next-word suggestions followed, and two
  quick spaces typed ". ".
* **Then every binding file** in `lib/rust/src/main/kotlin` (66 files, 3,078 lines).
  **Done (2026-11-21):** all 388 production native methods are generated, and a hand-written one
  now fails `checkNativeDeclarations`; ten mutations, one per new shape, each fail it too. 283
  hand-written lines are left in 13 files, none with an `external fun` (from 2,338 in 66; see
  [rust.md](rust.md#what-stays-hand-written)). Match tests are unchanged. On the emulator,
  "teh" composed with "the" offered and a space corrected it, next-word suggestions followed, one
  shift capitalised a letter, a double shift locked caps, two quick spaces typed ". ", a
  long-press opened a popup, and the Theme, Smartbar and Typing settings screens drew.
* **Exit:** all `external fun`s generated, at most about 300 hand-written lines left in the
  bindings, existing match tests unchanged.

### 3. Rust owns the keyboard (L)

Stage 5 packages 1–3, one event family at a time:

1. the typing-event fan-out (grade item A7);
2. the key and layout object graph (`TextKeyboard`, `TextKey`, the snapshot in
   `RustKeyboardView`);
3. theme, then subtype;
4. a plain-View root in place of the Compose root, proving lifecycle, insets, Snygg
   backgrounds, font changes and live inline autofill.

* **Exit for each:** 300-press and 8-popup digests, seeded 50,000-event traces with their
  ordered effects across cancellation and recreation, the reshow and reading-order probes,
  and the phone p95 again at the end.

**Item 1 done (2026-11-22)** ([phase 3: the typing-event fan-out](phase3-typing-fan-out.md)).

* **The typing plan.** Each typing event is one call to the NLP session, `NlpSession.event`,
  which returns one plan. The session counts, learns, teaches the tap trail, and arms or takes
  the auto-correction undo itself. Kotlin carries the event in and starts the save timers and
  the kept-word write the plan asks for.
* **The smartbar.** Each action's icon, label and flags, the arrow, the toggle's icon, and
  what the toggle flips are chosen in Rust.
* **The proof.**
  * Three seeded 50,000-event traces match main's frozen fan-out at every event, across save
    timers that fire or die with the process and service recreation.
  * 20,000 seeded smartbars match main, except for two label fixes, each counted.
  * The digests are unchanged, and the reshow and reading-order probes pass.
  * 41 of 41 mutations are caught.
  * CPU trials: the branch is no slower than main beyond noise. Neither met the 2 ms gate on a
    loaded host, so it is not a pass.
* **The size.** Production Kotlin is 29 counted lines smaller.
* **Owed.** P4 (the phone p95) stays owed, as do the new P22, E34 and E35.

**Item 2 done (2026-11-22)** ([phase 3: the key and layout object graph](phase3-key-graph.md)).

* **The key graph.** Rust reads the layout and popup-mapping files, builds the keys, and
  computes each one's label, code, icon, enabled state, popup set and hints for the keyboard's
  state (`lib/rust/crates/keygraph`). Kotlin reads the files, reports the facts only Android
  knows, answers the casing questions only Android's ICU can, and carries out what a key does.
  The Rust renderer draws its keys from this graph; the Compose fallback keeps the frozen Kotlin
  graph until step K.
* **The smartbar and its panels.** Each action's enabled state, icon and label are answered
  from the same graph, on the bar and in the overflow and editor panels.
* **The proof.**
  * Three seeded 50,000-state traces match main's frozen graph key by key and field by field,
    with no difference, across every mode, shift state, IME action, input class, right-to-left
    and non-Latin locales, synthetic layouts, abandoned computations and service recreation.
  * 20,000 seeded smartbars match main, except for item 1's two label fixes; their panels'
    answers match main's evaluator with no exception.
  * The digests are unchanged, "teh" and a space still type "the ", and the reshow and
    reading-order probes pass.
  * 73 of 73 mutations are caught, the last after a unit test was added for it.
  * CPU trials: the branch is no slower than main beyond noise. Neither met the 2 ms gate on a
    loaded host, so it is not a pass.
  * Layout rebuilds: mode switches are faster than main's, subtype switches 17 to 57 ms
    slower at p50 in both pairs of trials, which was owed a profile (E41, closed at item 3).
* **The size.** Production Kotlin is 305 counted lines larger: three new adapters, while the
  Kotlin graph stays for the Compose fallback. Its files are marked for deletion at step K.
* **Owed.** P4 (the phone p95) stays owed, as do the new P25 and E38–E40. (E41 was closed at
  item 3.)

**Item 3 done (2026-11-22)** ([phase 3: theme, then subtype](phase3-theme-subtype.md)).

* **The theme.** Rust chooses the active theme (day, night, follow the system or the clock, a
  preview), runs the theme session (what to load, keep or show again, and what a failed load
  shows), holds the base style as JSON, resolves the inline autofill chips' colours, and picks
  the palette's primary colour (`lib/rust/crates/imetheme`). Kotlin reads the settings and the
  system's night mode, unpacks and reads the stylesheet Rust names, keeps the loaded themes in the
  slots Rust names, and builds Android's inline suggestion style bundle. The Rust renderer no
  longer reads the Compose root's palette or the Compose chip cache.
* **The subtype.** Rust holds the subtype list and the active subtype
  (`lib/rust/crates/imesubtype`): it loads and repairs the stored list, picks the active subtype,
  answers the language key's next and previous and a switch by id, says which settings writes to
  make, and answers what each subtype names (its layouts per layout type, popup mapping, currency
  set, composer, NLP providers) with a cache key for its keyboards. The key graph's build row
  takes those facts from Rust instead of from the Kotlin `Subtype`. Kotlin names locale tags with
  Android's `Locale`, reports the currency sets it loaded, makes the Jetpref writes Rust asks for
  and publishes the flows the readers collect.
* **The proof.**
  * Three seeded 50,000-state theme traces match main's frozen theme manager at every state,
    across extension index changes, every theme mode, the clock, previews, failed and cancelled
    loads and service recreation; what the renderer is handed is equal too.
  * Three seeded 50,000-state chip traces match main's Compose query over the bundled
    stylesheets, the base style and 400 synthetic ones.
  * Three seeded 50,000-state subtype traces match main's frozen subtype manager and key-graph
    build row at every state, across the settings screens' edits, the language key, switches by
    id, duplicate ids, right-to-left locales, process death mid-flight and the keyboards' cache.
  * The digests are unchanged, "teh" and a space still type "the ", and the reshow and
    reading-order probes pass.
  * 42 of 42 theme mutations and 34 of 34 subtype mutations are caught.
  * CPU trials: neither half is slower than main beyond noise. Neither met the 2 ms gate on a
    loaded host, so it is not a pass.
  * The subtype switch, profiled with simpleperf (which runs without root on its software
    clock): the subtype's own work is under a millisecond a switch. Glide rebuilt its index on
    5 to 11 of 20 subtype switches, on main too, because the renderer paired the last subtype's
    keys with the new one; that is fixed, and a subtype switch now costs 48 to 62 ms less CPU
    than on main. A system trace puts the release's wait in agent-device's tap injector, before
    the event reaches system_server, and the frame after it waiting for the emulator's vsync.
    Mode and subtype switches were level with main in the final timings. So E41 and E44 are
    closed and the real timing goes to the phone (P30).
* **The size.** Production Kotlin is 330 counted lines smaller after the theme, and 83 larger
  after the subtype: the subtype's adapter and the glide fix.
* **Owed.** P4 (the phone p95) stays owed, as do the new P28, P30 and E46–E48.

**Item 4 done (2026-11-23)** ([phase 3: a plain-View root](phase3-view-root.md)).

* **The root.** With the Rust renderer on, the input view is `ImeRootView`, a `FrameLayout`, and
  the input method window holds no composition. Plain Views host the keyboard's surface, the
  window's background and image, the one-handed panels, the subtype sheet, the full-screen
  editor and the inline autofill chips. The theme, the settings and the managers' flows are
  collected for the view's lifetime. Switching the setting swaps the root in the same process.
  The frozen Compose root is the fallback until step K.
* **Rust decides.** Rust plans the window (`render::ime_root`): the input view's height, the
  panel showing, the safe insets, the bottom offset, the one-handed panels, full-screen's column
  and the surface's headroom. It also answers the window's background, image and navigation-bar
  flags (`imetheme::window`). Kotlin measures, decodes the image Rust names, and places views
  where the plan says.
* **Full-screen landscape.** A top-row key's popup and press bubble are no longer cut at the
  keyboard's top: the surface rises above the extract view while a finger is down or a popup is
  open.
* **The proof.** All 34 mutations of the plan and background are caught. A seeded differential over
  18 poses matched main's Compose root to the pixel in 14 to 15 of 18 per run; the two
  full-screen 3-button landscape poses differ as intended, and the rest is a 1 to 2 px flake on
  both builds (E53). The 300-press and 8-popup digests are unchanged, "teh" and a space types
  "the " on both renderers, reshow is 16 of 16 and the reading order passes. The proof found one
  real bug: a root swapped in at runtime got no insets and sat a navigation bar low. It is
  fixed, and the swap check now asserts the swapped-in root sits where a fresh one does.
  Creating the view takes 157 to 197 ms cold against 436 to 501 and 2 to 5 ms warm against 59
  to 61, the first frame comes about half a second sooner and PSS is about 4 MiB lower. The
  CPU trials' p95 is 1.992 to 2.374 ms against 2.959 to 3.522, but neither build meets 2 ms on
  this loaded host (the branch in one trial of five). See
  [phase 3, item 4](phase3-view-root.md).
* **The size.** Production Kotlin is 375 counted lines larger while both roots are in the build.
  That is well over the end state's 600 for the IME service, root, surface, touch and insets: the
  root's six files alone are 666.
* **Not done.** Icons were still Compose `ImageVector`s, flattened to paths, and theme fonts were
  not drawn, as on main; the Compose-free render path, below, did both. Live inline autofill was
  not seen on a device.
* **Owed.** P35, E52 and E53.

**After item 4: the Compose-free render path (2026-11-23)**
([phase 3: the Compose-free render path](phase3-compose-free-path.md)).

* **Icons.** Rust holds the 130 icons the renderer draws (`crates/icons`). It flattens them with
  ports of Compose's `toPath`, Skia's path, hwui's `approximate(0.25)` and Kotlin's float
  formatting. Kotlin sends a token or nothing. `RendererIcons`, `SettingsIcons` and
  `RustKeyPreview` are deleted.
* **The palette.** Rust makes the Material You scheme (`crates/palette`, a port of
  material-color-utilities by way of MaterialKolor) for every style, contrast and spec, light and
  dark, and `SnyggPalette` is deleted.
* **No Compose state or type.** The root reads StateFlows, `android.util.Size` and
  `KeyboardState.isRtl`.
* **Theme fonts.** The renderer draws a theme's `@font` files through parley, then today's chain.
* **The check.** `:app:checkComposeFreeRender` reads every compiled class in
  `com.jtn0123.audiowhisper.render` and fails on any that names `androidx.compose`. It runs in the
  gate, CI and the README's line.
* **The proof.**
  * **Icons:** all 130 icons (4,445 points) match what an Android device made through the old
    `iconPath`, bit for bit; so do the 38 key tokens.
  * **Palette:** 7,200,000 of 7,200,000 schemes match materialkolor on the JVM (50,000 accents ×
    72 flag sets × light and dark). On the emulator's ART, 144,000 of 144,000
    (1,000 accents).
  * **Fonts:** a fixture font theme draws in its font, a theme without fonts is byte-identical,
    and nine broken files fall back without a crash.
  * **Mutations:** 27 of 27 font mutations and 10 of 10 check mutations are caught.
  * **Probes:** the 300 presses and 8 popups type the same digests as main's build; "teh" and a
    space commit "the "; reshow 16 of 16; the reading order unchanged. Main's build and the
    branch's draw every panel with an icon identically on emulator-5558, by day and by night,
    apart from live GIF results and caret blink.
  * **CPU:** no change visible on a loaded host.
  * **The strip:** one strip change per press. The path's cheaper rebuild had drawn Rust's
    emptied row between a press's two lists more often; the strip now keeps its words while
    their refill is due, for at most 200 ms (E56, closed).
* **The size.** Production Kotlin 405 files and 31,810 counted lines, from main's 408 and
  31,970: four files deleted, one added.
* **Owed.** The whole path on the phone (P36); the 2 ms p95 on a quiet host (E28) and on the
  phone (P4). Step K
  still has to take the Compose keyboard, Smartbar and root out of the app
  ([phase3-compose-free-path](phase3-compose-free-path.md#what-remains)).

### 4. Settings (M–L, alongside 3)

[compose-removal.md](compose-removal.md) steps 5–12, in its order:

* **Backup and restore first.** It sets up the Rust file-picker boundary that dictionaries and
  extensions reuse.
  **Done (2026-11-21):** Rust draws Back up and Restore, and the Compose screens are gone
  (636 lines). Rust asks for the picker and decides what follows; Kotlin keeps documents under
  handles and runs the file work one job at a time. Every Compose backup restores as before,
  proven over all 63 group sets ([stage 4: Back up and Restore](stage4-backup-restore.md)).
* **Languages and dictionaries share editable lists.**
  **Languages done (2026-11-21):** Rust draws Languages & Layouts, Select language, the
  language pack manager and the subtype editor, and the Compose screens are gone (1,239 lines
  deleted, 177 added). The editable list and its editor are built for dictionaries to reuse.
  Rust writes the subtype list itself, proven byte for byte against the original codec
  ([stage 4: Languages and layouts](stage4-languages.md)).
  **Dictionaries done (2026-11-21):** Rust draws both user dictionaries on that list and
  editor, and UserDictionaryScreen is gone (542 lines deleted, 210 added). The words stay in
  Room and Android's provider, written with the same calls as before, proven against a frozen
  copy of the old code; old export files import, and a new export is byte-compatible
  ([stage 4: User dictionaries](stage4-user-dictionaries.md)). Since 2026-11-22 the Floris
  dictionary's words are in a Rust store, not Room ([phase 5: the stores](phase5-stores.md)).
* **The project licence.**
  **Done (2026-11-21):** Rust draws it, with read-only text selection on the editor's handles
  and floating toolbar, and ProjectLicenseScreen is gone (81 lines deleted, 123 added). Kotlin
  only reads the asset, sets the clipboard and shows Android's toolbar. TalkBack can select,
  copy, and read it by character, word, line, paragraph and page
  ([stage 4: the project licence screen](stage4-licences.md#the-project-licence-screen)).
* **The theme editor and extension editor are ported and improved**, not dropped (owner,
  2026-11-21). Each port says which improvements it makes, as the Other and Theme port did.
  **Theme manager done (2026-11-21):** Rust draws the day and night theme managers, with the
  keyboard previewing the chosen theme, and ThemeManagerScreen is gone (144 lines deleted, 63
  added). Five improvements, among them *Create* and *Import* on the manager and TalkBack
  hearing each theme's day or night kind. The editor itself is parts 2 and 3 of step 9
  ([stage 4: the theme manager and theme editor](stage4-theme-editor.md)).
* **Rust owns the preferences file** (`crates/settings` already has byte parity). Then Jetpref,
  then Room and KSP, leave the build.
  **Room left (2026-11-22)**, ahead of the preferences file: once phase 5 moved the clipboard
  history and the user dictionary to Rust, nothing used it. Its runtime, compiler, KSP options
  and schema files are gone. **KSP stays** for Jetpref's `@Preferences` processor, which
  generates `FlorisPreferenceModelImpl`, until Jetpref goes
  ([phase 5: the stores](phase5-stores.md#room-and-ksp-leave-the-build)).
* **Exit for each step:** as compose-removal.md says, plus real TalkBack on settings before
  step 12.
* **The Compose settings host.**
  **Done (2026-11-22):** `FlorisAppActivity` is a thin Rust settings host, and a link to a page
  Rust does not draw opens Rust's own not-found page instead of a blank Compose activity. The
  fallback activity, the navigation graph, the settings-only Compose primitives and
  `navigation-compose` are gone (18 files, 1,603 lines). Every way into settings is tested from
  the intent to the Rust screen it opens ([stage 4: the Compose settings host](stage4-compose-host.md)).
  Step 12 went ahead before the TalkBack exit above, which is phone check P6: since step 10 the
  fallback drew no screen, so nothing it deleted was a screen TalkBack could fall back to. P6
  stays owed.

### 5. Platform edges (M)

* **The clipboard store moves from Room to Rust.**
  **Done (2026-11-22)**, with the keyboard's own user dictionary. `crates/stores` owns the format,
  the files and every change: one checksummed, versioned file per store, saved crash-safely.
  Kotlin keeps the system clipboard listener, the media files' bytes and the content provider,
  which asks Rust for each record. Main's Room databases are imported once, at every schema main
  shipped, and deleted, safe against a crash at any step. Proven against main's Room by 1,800
  replayed steps, on every schema by fixtures main's build wrote, and on the emulator from main's
  build with data to this one ([phase 5: the stores](phase5-stores.md)).
* **Decisions move to Rust:** the GIF request lifetime, the sequencing and fallback decisions
  of dictation and read-aloud, crash handling, and locale. These are stage 5 packages 4–6.
  **Done (2026-11-22) for GIF, dictation, read-aloud and crash; locale is separate.** Rust runs
  the GIF panel's session, the microphone's reads and the model check, every step of a reading
  and the neural voice's lifetime, and what a crash leaves behind and the report it shows.
  Kotlin keeps the network, ImageDecoder, URI grants, AudioRecord, AudioTrack, the voices, audio
  focus, the AccessibilityService and the crash files. Proven by seeded traces against main's
  code (50,000 events a seed), 123 mutations and checks on the emulator. With the native library
  failing to load, the crash path now reaches the platform crash screen; on main it ended the
  process ([phase 5: the edges](phase5-edges.md)).
* **Exit:** stored-data migration fixtures and emulator flows.

### 6. Retire the references (M, C++ half done)

* **The C++ oracle and the frozen `Reference*.kt` test copies become fixed test data** in cargo
  tests (owner, 2026-11-21). The `Reference*.kt` copies are 148 files, about 17,400 lines.
* **Then they are deleted:** `lib/latinime`, `latin-oracle`, `oracleCheck`, and each Kotlin
  reference once the wrapper it guards is gone.
* **Proof:** each set of fixed data catches the mutations in the existing mutation scripts
  before its source is deleted.
* **Done, the C++ half (2026-11-21):** the oracle's answers are golden files in the `latin` and
  `suggest` crates, checked by the everyday `cargo test`; they kill every mutation of
  `scripts/latin-mutations.py` the oracle killed; `lib/latinime`, `latin-oracle` and
  `oracleCheck` are deleted ([record](phase6-oracle-goldens.md)).
* **The Kotlin half, begun (2026-11-23):** the inventory of all 166 `Reference*.kt` files, with a
  verdict for each, is in [phase6-kotlin-references](phase6-kotlin-references.md). Dictation,
  reader and gif are retired: 23 copies became 40 golden files (22 KB) checked by the JVM tests
  that compared against them, and they catch every mutation the copies caught. The 7 copies no
  test read are deleted too. 19 device comparisons stay (their goldens need a device run), as
  do the 8 render references and 2 Snygg palette references (in flight, the Compose-free render
  path) and the 5 the render references build on.
* **The Kotlin half, second branch (2026-11-23):** suggest, nlp, editor/input and keyboard are
  retired: 38 copies became 73 more golden files (42 KB), and the goldens catch every mutation
  the copies caught, and one more (`habit-weight-at-the-boundary`). Kept from these families:
  `ReferenceKeyboardSubtypes`, whose trace drives a render reference, and the editor drivers'
  platform stand-ins (`ReferencePlatform`, `ReferenceEditorRange`, `ReferenceCaps.java`).
  The lead kept three more Snygg copies with the palette's two.
* **The Kotlin half, the device comparisons (2026-11-23):** the 19 copies the instrumented tests
  compared against are retired. Their answers are 17 golden files (6.6 KB): 11 written on an
  emulator into the test APK's assets, and a1's and the extension maintainer's on the host. Each
  family's device fault is caught before and after; whether a phone gives the emulator's speech
  goldens is owed ([owed-checks P37](owed-checks.md#phone-checks-the-docs-owe)).
* **Left, the Kotlin half:** app, lib, rust boundary, settingsui, theme and snygg: 58 copies,
  each retired the same way.

## Risks

| Risk | What to do |
|---|---|
| Performance | The 2 ms gate has never run on the phone. Measure in phase 1 before deleting more, and again after phase 3, which should cut crossings. |
| Accessibility | The biggest product risk: a custom surface with no real TalkBack session yet, and focus and hover policy still in `RendererAccessibility.kt`. Real TalkBack on the keyboard (phase 1) and settings (before step 12); add an accessibility-tree golden to the emulator scripts. |
| Android API churn | Hand-written entry points keep deprecations visible to lint; test physical API behaviour on each SDK change. |
| Reference rot | The decoder is tuned away from AOSP over time. The C++'s answers are fixed as goldens (phase 6); fix the Kotlin references' while they still match. |
| Build complexity | The generator adds a host cargo build before Kotlin compiles. Output in `build/`, a `--check` in the gate, and fix the build-time GitHub dependency (grade item F3). |
| Test coverage | Most JVM tests run through wrappers that phase 3 deletes. Move each proof to cargo in the same commit as its wrapper, and only after the new test fails a mutation. |
