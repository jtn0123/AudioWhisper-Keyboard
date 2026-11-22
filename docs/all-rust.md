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
  `stage6/*` branches were fully merged and are deleted.
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
  ([stage 4: User dictionaries](stage4-user-dictionaries.md)).
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
* **Exit for each step:** as compose-removal.md says, plus real TalkBack on settings before
  step 12.

### 5. Platform edges (M)

* **The clipboard store moves from Room to Rust.**
* **Decisions move to Rust:** the GIF request lifetime, the sequencing and fallback decisions
  of dictation and read-aloud, crash handling, and locale. These are stage 5 packages 4–6.
* **Exit:** stored-data migration fixtures and emulator flows.

### 6. Retire the references (M)

* **The C++ oracle and the frozen `Reference*.kt` test copies become fixed test data** in cargo
  tests (owner, 2026-11-21). The `Reference*.kt` copies are 148 files, about 17,400 lines.
* **Then they are deleted:** `lib/latinime`, `latin-oracle`, `oracleCheck`, and each Kotlin
  reference once the wrapper it guards is gone.
* **Proof:** each set of fixed data catches the mutations in the existing mutation scripts
  before its source is deleted.

## Risks

| Risk | What to do |
|---|---|
| Performance | The 2 ms gate has never run on the phone. Measure in phase 1 before deleting more, and again after phase 3, which should cut crossings. |
| Accessibility | The biggest product risk: a custom surface with no real TalkBack session yet, and focus and hover policy still in `RendererAccessibility.kt`. Real TalkBack on the keyboard (phase 1) and settings (before step 12); add an accessibility-tree golden to the emulator scripts. |
| Android API churn | Hand-written entry points keep deprecations visible to lint; test physical API behaviour on each SDK change. |
| Reference rot | The decoder is tuned away from AOSP over time. Fix the goldens (phase 6) while they still match. |
| Build complexity | The generator adds a host cargo build before Kotlin compiles. Output in `build/`, a `--check` in the gate, and fix the build-time GitHub dependency (grade item F3). |
| Test coverage | Most JVM tests run through wrappers that phase 3 deletes. Move each proof to cargo in the same commit as its wrapper, and only after the new test fails a mutation. |
