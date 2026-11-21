# Rust in the keyboard

AudioWhisper Keyboard is moving its own logic from Kotlin to Rust, one piece at a time. Each
piece lands only when it gives the same results as the Kotlin it replaces, proven by a test that
runs both, and has been checked on the emulator.

## Rust is the default; Compose is frozen (2026-11-16)

Rust draws the keyboard and settings by default; a user's explicit "off" is kept. Compose gets
no new features, only fixes for crashes or lost data, and is removed in the order of
[the removal plan](compose-removal.md). The Compose keyboard stays as the fallback until its
phone checks pass (a real TalkBack session, the Samsung font check, font scale 2.0 and the CPU
p95 gate, which the emulator has passed since 2026-11-17 and the phone has yet to; see
[the results](cpu-p95-results.md)). Where the stages below say "Compose stays the default" or
"default-off", that was true when they were written. See [decisions](decisions.md#rust-draws-by-default-2026-11-16).

## Logic and drawing in Rust (decided 2026-11-14)

The goal is **logic in Rust**: every decision and all UI drawing belong in Rust. Kotlin
and Java remain only as thin Android adapters. An adapter transports platform facts,
executes a Rust plan and returns the actual Android result; it does not choose policy,
validate domain data, own editing/navigation state or draw application controls.

Android adapters include IME/Activity/service lifecycle, InputConnection, surfaces and
Choreographer, insets, accessibility transport, resources/system fonts/colours, file/network
IO, audio, Room/content providers, intents, SAF and permissions. Platform-owned UI such as
the IME picker, permission dialogs and inline autofill views remains Android-owned. Rust
chooses the requests, layout, semantics and actions; Android supplies platform capabilities.
Keep genuine Android APIs and move decisions out of their wrappers rather than replacing
each Kotlin line with a JNI call. The full settings port and fallback deletion still require
their separate acceptance gates.

**History:** on 2026-11-12 the user replaced the original logic-only boundary with a target
of at least 90% Rust, explicitly including keyboard drawing and settings. On 2026-11-14 the
user replaced that percentage target with the ownership goal above: stage 4 sizing showed
that settings plus Compose keyboard deletion reaches only about **63–67% Rust**. A line
quota would obscure the remaining policy work and penalize necessary Android adapters.
Report Rust share after each package; the percentage is descriptive, never a target. Never
add Rust or remove tests to improve it.

Count non-blank lines of tracked `.kt`, `.java` and `.rs` files; exclude paths containing
`/test/`, `/tests/`, `/androidTest/` or `benchmark/`, `*Test.kt`, `*_test.rs`, `*_tests.rs`,
`tests.rs`, generated tables (`latin/src/chars/tables.rs`, `chars/src/tables/*`), and the
entirely test-only `latin-oracle` crate. Dependencies' native source is not ours and does not
count, including sherpa-onnx, onnxruntime and renderer dependencies. Preserve the supplied
counter's separately documented latin-oracle discrepancy when reporting its result.
The historical planning snapshot was about 25.8k Rust, 55.8k Kotlin and 0.6k Java (31%).
After stage 1 the 2026-11-12 recount was 25,306 Rust, 53,808 Kotlin and 627 Java (31.7%).
Stage percentage headings below are historical estimates, not acceptance targets.

### Stage 1: remove features we do not use (done)

Developer tools and non-Latin input were removed, including the Han provider and pack,
non-Latin composers, layouts, popup mappings and presets. This makes the next ports smaller,
but moves the percentage only to about 32%. Latin-script layouts, accented letters,
Vietnamese Telex, numeric/symbol/phone layouts, emoji, add-ons and the theme editor stay.

### Stage 2: remaining logic (roughly 50% Rust)

Port these packages in order, retaining a test-only Kotlin reference for each. Package sizes
are planning estimates, not delivery dates. Android ICU segmentation and locale casing stay
until a replacement matches Android. Keep Android work at the edge and avoid redundant JNI
wrappers: the point is to remove Kotlin decisions, not translate each line through a call.

1. Key/domain models: selectors, codes/types, currency slots, keyboard state and text-key evaluation (done; stage 2.1 below).
2. Layout assembly/cache: arrangements, merging, hints and cache keys (done; stage 2.2 below).
3. Snygg rules and stylesheet grammar: parsing, ordering and schema/version errors (done; stage 2.3 below).
4. Snygg values: dimensions, colours, fonts, padding, shapes and variables (implemented; stage 2.4 below, emulator acceptance blocked).
5. Snygg compilation/editor model: cascade, inheritance, matching and editor operations (done; stage 2.5 below). The extensible JVM schema DSL/generator stays at the Kotlin boundary.
6. A1: native Rust suggest/decoder session; production LatinIME Java glue removed, C++ oracle retained (done; [proof](stage2-a1.md)).
7. Emoji history and quick-action arrangement (done under stage 3B; [package 6 proof](stage3b-package6.md)).
8. Clipboard ordering, duplicate/expiry/limit/pinning and MIME policy; Room and the Android clipboard stay in Kotlin (done under stage 3B; [package 7 proof](stage3b-package7.md)).
9. Extension metadata/import policy: identifiers, maintainers, validation and archive-entry decisions (done; [proof](stage2-extensions.md)).
10. Dictation session state machine, compared with the existing schedules (done implementation and dictation proof; [proof and final device-acceptance hold](stage2-dictation.md)).
11. ASR and VAD through sherpa-onnx 2.0.1's exact C API; restore its excluded C library and preserve audio/decoder/VAD settings and flushing (done implementation and byte-identical dictation proof; [proof and final device-acceptance hold](stage2-dictation.md)).
12. Pocket TTS through that C API: Loona, four threads, cancellation and stop-before-free; sherpa JNI removed from normal APKs (implemented; local proof and explicit device-acceptance hold in [package 12](stage2-tts.md)).
13. Input dispatch and KeyboardManager routing: press/release/repeat, pointer policy, actions, state, evaluator and feedback (done; [proof](stage2-input.md)).
14. Editor state/reconciliation: expected text, selection/composing and UTF-16 (done; [proof](stage2-editor.md)).
15. Editor operation plans: deletion, replacement and commits over a fake InputConnection (done; [proof](stage2-editor.md)).
16. NLP orchestration: provider eligibility, scheduling/generations, candidate and inline state, learning and smartbar plans (done; [proof](stage2-nlp.md): 50,000 orchestration events/200 seeds, 50,000 learning events/200 seeds, real dictionary traces, restored mutations and both-renderer emulator comparisons).
17. Subtype/settings defaults, validation, migration and active subtype (done code/JVM proof; 143 defaults, every historical key family, 30,000 subtype operations/200 seeds, generated stored-file byte parity and restored mutations; [proof and device-acceptance hold](stage2-settings.md)).
18. Backup/restore manifests, conflicts and error classes (done code/JVM proof; [proof and device-acceptance hold](stage2-backup-utils.md)).
19. Remaining utility rules: MIME matching, formatting and validation (done code/JVM proof; [proof, platform boundaries and device-acceptance hold](stage2-backup-utils.md)).
20. Consolidate engine boundaries, removing Kotlin–Rust callback chains and redundant wrappers (done; [inventory, proof and measurements](stage2-consolidation.md)).

### Stage 3: Rust draws the keyboard (roughly 68–76% Rust)

Kotlin hosts a SurfaceView in a FrameLayout under the IME service, maintains insets and
touchable regions, and forwards complete packed MotionEvents: action, pointer ids, coordinates,
history, timestamps and cancel. It keeps inline autofill views, GIF fetching/decoding and an
AccessibilityNodeProvider. Rust supplies the TalkBack semantic snapshot (stable ids, bounds,
labels, roles and actions), following AOSP LatinIME's accessibility delegate.

Rust also decides the order a screen reader reads, in `render/src/reading_order.rs`: the order
a sighted person scans. The strip above the keys comes first (the Smartbar's buttons and words,
or the typed-text preview, or the search strip), then the keys or the open panel. Each reads
row by row, top to bottom, and each row left to right as drawn. A popup's choices read as one
block straight after the key or item that opened it: a held key's accents, an emoji's variants,
a clipboard item's menu, the clear-history dialog after its button. A backdrop or scrolling area
reads just before the first thing inside it. Each node names the one read before it, and Kotlin
passes that to Android as a traversal hint. Each node also keeps its drawing layer, so a touch
still lands on what is drawn on top, such as a popup over the keys.

Rust takes ANativeWindow through JNI, owns a render thread with surface-generation ids, and
renders only dirty or animating frames using Choreographer timestamps. It owns the scene,
hit testing, keys, smartbar, candidates, panels, glide trail and popups inside an enlarged
surface. Existing Snygg JSON themes remain the input; density, font scale and Material You
colours come from Android. The theme editor hosts the same renderer through AndroidView and
passes draft snapshots, so previews and the keyboard use the same rules.

Try Vello (hybrid over wgpu/Vulkan or CPU, chosen by measurements), Parley and Skrifa first.
Keep system fonts and emoji, loaded through public APIs. First inspect the emulator and S24
fonts' CBDT/CBLC or COLRv1 format; prove bitmap/colour emoji, ZWJ families, flags, skin tones,
keycaps and VS15/VS16 work. skia-safe is the fallback if pure Rust cannot match appearance or
speed; it brings Skia C++ and an estimated 15–35 MiB, so measure its actual cost before choosing.

The prototype stays behind a setting with Compose as fallback. It must show QWERTY normal,
pressed and shifted labels, a suggestion strip with colour emoji, long-press e accents, a
glide trail, one bundled light and dark theme and a live theme-editor change. Exercise surface
recreation, rotation, cancel and a minimal TalkBack key list. Accept only identical committed
text and action traces from recorded input, plus reviewed visual pairs within anti-aliasing
tolerances. Target warm CPU scene p95 <= 2 ms and GPU p95 <= 3 ms. Use Perfetto FrameTimeline
for touch-to-present, missed frames, memory and idle power; measure added native-library size.
Alternate Compose/Rust trials first on the emulator, then on the S24 Ultra at 120 Hz. Only
after phone acceptance remove the Compose keyboard, smartbar/candidates/popups, media/emoji/
clipboard panels and Snygg UI; this prevents the fallback becoming permanent duplicate code.

#### Milestone A prototype (2026-11-12)

The default-off **Rust keyboard renderer (prototype)** setting draws character keys,
candidates and evaluated accent popups through Vello CPU. A Vulkan/Hybrid measurement path
is opt-in. On `aw_keyboard` (Android 16, 60 Hz), Parley/Fontations rendered the system's
COLRv1 emoji and CBDT/CBLC flags, including ZWJ, tone, keycap and VS16 sequences. Compose,
CPU and Hybrid match both committed-text digests in the fresh 300-press/eight-popup
trials. Separate strip/key damage reduces CPU frames over 2 ms from 42.9% to 10.1%;
two alternating final CPU scene+raster trials have p95 4.715 and 5.533 ms, versus
7.809 and 8.479 ms before. Candidate edge cancellation, fresh light/dark/popup captures,
and portrait/landscape recreation are checked on the final APK; window ownership ends at
zero (5 acquired/5 released). **The 2/3 ms performance gate is not passed.** (2026-11-17: the
CPU half now passes on the emulator, p95 1.257 and 1.107 ms; see
[the CPU gate results](cpu-p95-results.md). The phone is still owed, and the 3 ms GPU target
applies only if the build-option Vulkan path is measured again.) Compose stays
the default and the prototype was left off. Milestone B still needs Samsung font inspection,
S24/120 Hz Perfetto and GPU timings, broader input parity and closer visual matching.
See [the full results and reproduction notes](stage3a-results.md) for boundaries and evidence.

`rendererEmojiSamples()` in CandidatesRow and `traceComposeFrame()` in TextKeyboardLayout
are temporary measurement scaffolding to remove with the Compose keyboard. With their
`aw-emoji` and `aw-compose` log tags unset they supply no samples and register no frame
callbacks; the default-off prototype leaves the existing Compose path in use. The emoji
hook only supplies fixed synthetic samples when real candidates are absent and logs nothing.
The frame hook logs numeric touch/commit timestamps only, never words or candidate text.
`log.tag.aw-profile=DEBUG` at native surface creation opts into CPU attribution: damage
bounds and exact union paint pixels, full/dirty, press/release, surface creation, case-change
(shift) proxy, popup/candidate changes, shaping misses, glyph calls, raster and buffer
lock/copy/post time. The profile tag also enables numeric Kotlin surface lifecycle events.
Those intervals overlap (shaping, glyphs and allocations are parts of encoding), and
allocation counters do not cover library internals; cold-layout glyph timing is a proxy,
not Vello's private atlas-miss count. No scene labels, candidate text or typed contents
enter measurement logs.

#### Milestone B package 1 (2026-11-12)

The prototype now compiles and resolves the active Snygg source in Rust, including extension
themes and named Material You palettes, with cached drawing states and raw theme inputs
across JNI. All 12 bundled themes match the test-only Kotlin reference; six fresh visual
pairs and live theme switching are checked. The 2/4-worker Vello CPU experiment regressed
both p50 and p95 and was removed. Compose stays the default. See
[package 1 results, screenshots and per-trial measurements](stage3b-results.md).

#### Milestone B package 8 (2026-11-13)

Subtype selection, one-handed side controls, incognito watermark, landscape extract chrome
and dictation preview now draw in Rust. ExtractEditText remains an Android View, and the
settings key preview uses offscreen Vello CPU pixels. Live editor stylesheet copies now
refresh native JSON. Compose still hosts root/theme/state/resources and inline Android
views; the prototype stays default-off pending phone acceptance. The deletion dry run
removes 35 fallback files plus bypassed branches and compiles without committing deletion.
See [package 8 inventory, checks, timings and deletion counts](stage3b-package8.md).

#### Milestone B package 7 (2026-11-13)

Clipboard history, GIF/media results and GIF search typing now remain on the native
surface. Rust owns clipboard policy, panel geometry/menus/selection, visible animation
clocks and bounded pixel caches. Kotlin retains Room, Android clipboard, network/decoding
and existing rich-content commits. The prototype stays default-off. No GIPHY key is
configured: no-key parity and generated local GIF fixtures are checked; live GIPHY results
are unverified. See [package 7 results](stage3b-package7.md) for checks and limitations.

#### Milestone B package 6 (2026-11-13)

Emoji, editing quick-action tiles, overflow and the action-order editor now draw and handle
input on the Rust surface. Emoji history and arrangement policies also moved to Rust with
Kotlin reference match tests and deliberate mutations. Clipboard history and GIF/search
remain Compose; the prototype stays default-off. See [package 6 results](stage3b-package6.md).

#### Shown again quickly, drawn blank (fixed 2026-11-17)

Hiding the keyboard and showing it again within about three quarters of a second drew the
background and no keys. Android hides the keyboard in two steps. First it tells the keyboard's
views, not the window, that the window has gone, and the SurfaceView the keys are drawn on
gives up its surface. Half a second later it really hides the window. Showing the keyboard
only shows the window again, and a window that never really hid tells its views nothing. So
the SurfaceView went on believing its window had gone and never asked for a new surface: no
native window, no frames, until the next slow hide. The background survived because its
SurfaceView is removed and added back each time, and a view added to a window asks the window
itself. Compose draws into the window, not its own surface, so it never showed the fault.

The rule now: when the keyboard is shown, an input view last told its window was anything
but visible, in a window that is visible, is told again that it is visible
(`awnative/src/ime_window.rs`; `InputViewVisibility` keeps what it was last told, and
`FlorisImeService.onWindowShown` asks). A window that really hid is not visible yet when it is
shown, and Android tells the views itself once it is. `scripts/render-benchmark/reshow.py`
hides and shows the keyboard on the emulator four ways at four gaps and checks each time that
a new native window drew a frame and the key rows match a settled screenshot.

On the aw_keyboard_ssd emulator (1080x2400, Android 16), the build before the fix drew its keys
in 7 of 16 trials: every path came back blank at 300 and 600 ms, and the app switch at 100 ms
too. With the fix, 16 of 16. A 100 ms hide by the action, Back or an outside tap is too quick
for Android to release the window at all, so those keep their keys either way. Rotation, the
long-press popup, suggestions and switching the Rust drawing off and on again were checked on
the fixed build afterwards. Not yet checked on the phone.

### Stage 4: Rust draws settings (decision 2026-11-14)

Rust draws the settings Activity on the keyboard's Vello CPU / Parley / Skrifa stack.
This replaces the earlier plan to retain accessible Compose settings controls. Keep every
screen, add-on/extension flow and the theme editor. Rust owns navigation, screen descriptions,
widgets/layout, scrolling/fling, animation, hit testing, focus, semantics and editable models.
Kotlin retains a thin Activity/surface/input/back host, the Android accessibility bridge,
InputConnection endpoint, resources, file IO and execution of platform plans (SAF, intents,
permissions, IME picker and toasts). Android string resources remain the source of truth.

Compose stays the default and fallback behind a separate default-off settings renderer switch
until phone acceptance, just as for the keyboard. The current checkpoint ends after design,
a Home/full-settings-screen/text-field/platform-action prototype and the same stored-file,
reference/mutation, screenshot, accessibility and measured-runtime proof as other packages.
Unported routes continue in Compose. See [the complete design and inventory](stage4-settings.md).

The default-off Home/Keyboard prototype is implemented; [checkpoint results](stage4-settings-results.md)
record passing local comparisons/mutations and the device-tool hold. Final on-device stored-file,
TalkBack, visual, lifecycle and performance acceptance remains incomplete. Neither fallback is deleted.

The 2026-11-14 ownership decision replaces the 90% target. The settings scope plus the
measured keyboard deletion is projected at roughly 63–67% Rust; report that estimate without
chasing it. Inventory remaining models/wrappers/hosts for decisions and drawing that must
move to Rust, and name the Android API for each adapter that stays. The [stage 5 inventory](stage5-inventory.md) is research and
planning until separately authorized. Recount with the supplied source tool and refresh the
deletion dry run before claiming an endpoint. Neither Compose fallback is deleted here.

## What is in Rust

| Piece | Rust | Replaced | Proof it matches |
|---|---|---|---|
| Settings renderer prototype (stage 4 checkpoint) | `render::settings_ui`, `awnative::render_settings` | Home/Keyboard layout, controls, scroll, focus, semantics, UTF-16 scratch editor and saved native drafts behind a default-off selector; other screens use Compose | Frozen screen decisions, Compose EditProcessor differential checks, real host Jetpref byte parity, four restored mutations and native input/geometry tests. Device acceptance incomplete; see [checkpoint](stage4-settings-results.md) |
| Settings and subtypes (stage 2 package 17) | `crates/settings` | All preference defaults, scalar/enum admission, UI ranges, complex stored codecs, historical migrations, subtype edit/active/fallback/preset policy and settings derivations; Kotlin retains typed Jetpref I/O, Locale, resources and Compose | Frozen Kotlin references, all 143 preferences/27 ranges, bundled and seeded codecs, 1,000 historical states, 2,000 action migrations, 30,000 subtype operations/200 seeds and 20 real generated Jetpref files; restored default/bound/migration/fallback mutations. Device hub did not answer; upgrade/UI/typing acceptance remains owed. See [proof](stage2-settings.md) |
| Backup manifests, restore plans and utility rules (stage 2 packages 18/19) | `crates/backup`, `crates/utilities` | Archive layout/selection/metadata, ordered restore/conflict effects; MIME policy, ref normalization/classification, locale tag/display layout, versions/changelog and dictionary validation | Frozen Kotlin reference matches, locally generated ZIPs, exhaustive selection/presence/conflict masks, seeded and full UTF-16 validation, four restored mutations and full local gate. Device hub did not answer; UI/Android acceptance and the prior email assertion remain held. Existing learned-store backup exclusion is preserved. See [proof](stage2-backup-utils.md) |
| Pocket TTS and remaining reader queue/walk policy (stage 2 package 12) | `crates/reader`, `crates/sherpa-ffi` | C generator/PCM leases/cancellation, play/system-TTS queues, full screen selection/order, follow timing; Kotlin retains Android playback/focus/accessibility/files | Seeded queue/tree matches, retained chunk/follow tests, native fake-owner ordering, restored mutations; normal APK has no sherpa Kotlin/JNI. **PCM parity, real-model stress/timing, UI and final device acceptance remain unverified after two T3 shutdown failures.** See [package 12](stage2-tts.md) |
| Dictation session and sherpa ASR/VAD (stage 2 packages 10/11) | `crates/dictation`, `crates/sherpa-ffi` | Lifecycle/generations/timeouts, UI and editor plans, model policy, VAD segmentation and native recognizer/stream/cancellation ownership; Kotlin retains Android I/O/executors | 50,000 session events and 50,000 PCM chunks over 200 seeds each; recorded traces; seven byte-identical fixture comparisons (26 transcript hashes, 315 VAD observations, five segments, 14 partials and final flush); four restored mutations and four UI pairs; final legacy-input/cleanup/logcat/shutdown acceptance remains on a documented device hold. See [dictation proof](stage2-dictation.md) |
| Input dispatch and KeyboardManager routing (stage 2 package 13) | `crates/input` | Lifecycle/timers, shared pointer policy, ordered key routes, state/subtype transitions, evaluator tokens and feedback decisions; Kotlin executes Android effects for both renderers | 50,000 events across 200 seeds and 12,906 editor operations match frozen Kotlin; four restored mutations; existing 43,200-event touch comparison retained; pre/post Compose and Rust fixtures match. See [input proof](stage2-input.md) |
| Editor state, flags, reconciliation and operation plans (stage 2 packages 14/15) | `crates/editor` | Expected UTF-16 windows/queue, composing/selection, raw EditorInfo interpretation, commits/deletes/reverts/spacing/caps, batch sequences and editing routes; Kotlin interprets plans and supplies Android ICU/provider data | 50,000 seeded events over 200 seeds, 201,611 ordered effects and 93,887 checkpoints; five restored mutations including the batch-balance invariant; pre/post Compose and Rust regression, suggestions and extended editor fixtures match. See [editor proof](stage2-editor.md) |
| Extension metadata and import policy (stage 2 package 9) | `crates/extensions` | Identifier/maintainer/validation, type/target/skip and archive decisions; Kotlin keeps Android I/O, serialization and Compose resources | Seven bundled manifests, 155 document comparisons, exhaustive/seeded validation and maintainer matches, 10,000 entry lists and 11 generated ZIPs; ID mutation caught; Android ICU 302,152 matches and real valid/two-invalid import UI checks. See [extension proof](stage2-extensions.md) |
| Native typing decoder session (stage 2 package 6) | `crates/suggest` (`session`) calls `crates/latin` directly | Production LatinIME Java glue and Kotlin session/slot/geometry/result marshalling | Saved JVM/device 4,000 decoder and 2,000 full-pipeline states match byte for byte; 4,000 independent C++ oracle queries; result-order mutation caught; both renderer regressions and suggestion-pick/autocorrect digests match. See [A1 proof](stage2-a1.md) |
| Clipboard history ordering, duplicates, expiry, limits, pins and MIME eligibility (stage 2 package 8) | `crates/render` (`clipboard_policy`) | `ClipboardManager` / `ClipboardHistory` policy; Room and Android clipboard remain Kotlin | `ClipboardMatchTest` against test-only `ReferenceClipboard`, 12,000 cohorts and MIME combinations, deliberate mutations; paired paste/pin/unpin/delete/clear stored-state fixture. See [package 7](stage3b-package7.md) |
| Snygg compilation, cascade and editor model | `crates/snygg` (`resolve`, `editor`, `spec`, `json`) | `SnyggTheme` compilation/querying, property application/inheritance, built-in declarations and the theme editor's rule ordering/rename policy | JVM reference matches: 868,680 bundled-theme queries, 30,000 seeded queries and 7,200 editor steps; native JSON-to-style light/dark prototype oracle. Reversing cascade traversal fails. Five visual pairs are identical and editor edits persist; see stage 2.5 |
| Snygg values, specs and ordered property decoding | `crates/snygg` (`value`, `value_spec`, `value_codec`, `value_uri`, `properties`) | All 28 built-in encoders, defaults, validation, canonical formatting, encoder choice and property decoding policy | Three JVM value match suites against test-only `ReferenceSnygg*`, all 12 bundled themes and 28 inline documents, seeded/mutated and every UTF-16 unit cases; dividing percentages by 99 fails. Emulator acceptance is blocked by the device helper; see stage 2.4 |
| Snygg rules, selectors, attributes and ordered stylesheet structure | `crates/snygg` | `SnyggRule` and `SnyggStylesheet` grammar/formatting/comparison/schema policy, behind the same managed API | `SnyggGrammarMatchTest` and `SnyggStylesheetMatchTest` against test-only `ReferenceSnygg*`: all 12 bundled files, 28 inline fixtures, every configuration flag combination, UTF-16 and seeded/mutated cases. Reversing selector comparison fails. See stage 2.3 below |
| Layout selection and assembly: subtype/mode load plans, explicit modifier types, arrangement insertion, number/symbol hint alignment and computed-keyboard cache keys/invalidation | `crates/keys` (`assembly`) | `LayoutManager`'s decisions and merge/hint loops, plus `TextKeyboardCache`'s policy | `LayoutAssemblyMatchTest` against `ReferenceLayoutAssembly` and `ReferenceTextKeyboardCache`: every bundled layout, mode and retained preset, complete evaluated key/popup/hint lists and bit-exact geometry, seeded malformed/missing references and cache histories. Changing the insertion marker from code 0 to code 1 fails both layout tests. See stage 2.2 below |
| Key/domain models: selector choices, canonical key codes and predefined types/labels, currency slots, state bits, key strings, popup-source precedence, phone hints and enabled/visible policy | `crates/keys` (`codes`, `model_data`, `models`) | `KeyData` selectors, `CurrencySet`, `KeyboardState` bits, `TextKeyData` tables/string rules, and the pure parts of `TextKey`/`ComputingEvaluatorImpl` | `KeyModelsMatchTest` and `KeySelectorsMatchTest` against the test-only `ReferenceKey*` and `ReferenceKeyboardState`; every bundled layout in all selector states, seeded nested selectors, raw registers and transitions, currency sets, key/string and popup/availability cases. Reversing the unshifted case choice in Rust failed both selector tests. See stage 2.1 below for counts and emulator proof |
| Glide typing's scorer: pruning a swipe's candidate words and ranking them | `crates/glide` | `StatisticalGlideTypingClassifier`'s `Pruner`, ideal gestures and scoring | `SwipeIndexTest` against `ReferenceGlideClassifier`; on the emulator, 512 of 512 scorings were identical |
| Dictation's text rules: spoken commands, joining phrases transcribed separately, and taking back the last one | `crates/dictation` | `Spoken.parse`, `Phrases.join` and `punctuate`, `DictationUndo.revertAt` | `DictationRulesMatchTest` against `ReferenceDictation`: every UTF-16 unit through each rule, and 100,000 random phrases. The same cases on the emulator: none of 1,579,648 differed. Dictated recordings typed the same text |
| Dictation's filler words and custom vocabulary, new settings (Settings > Dictation): the Mac app's rules, not a port of the keyboard's own. The keyboard's spoken commands, joining and capitals stay as they were; a phrase is tidied before they see it | AudioWhisper-Core's `transcript`, a pinned git dependency of `crates/awnative` | Nothing replaced: `DictationRules.format`, called from `Dictation.type` | The Rust is the Mac app's, proven against its Swift there (AudioWhisper-Core's differential tests). Here `DictationRulesTest` checks the JNI: fillers, terms, CamelCase splits, `heard -> written`, comments, and text that isn't valid UTF-16 left alone; it fails for each of 2 breaks in the JNI. The page renders on the emulator |
| The mic key's tap and hold: a tap starts dictation and the next stops it, a hold of 300 ms or more dictates until let go, a press while listening finishes on release, and a release with no press seen toggles. Kotlin still starts and stops dictation | `crates/dictation` (`mic_key`) | `MicKey`, now a wrapper | `MicKeyMatchTest` against `ReferenceMicKey`: 2,000 random runs of 40 presses and releases in every status, at gaps either side of a hold and with clocks where a Long wraps. It fails for each of 18 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 200,000 differed. A press and release takes 0.10 µs against Kotlin's 0.04, the cost of crossing into Rust, far below a finger's time. A tap and a hold both work on the emulator |
| Typing suggestions' smaller rules: a word's case, the words before it, which characters continue a word, whether a correction is sure enough to make by itself, undoing one, and where taps land on the decoder's QWERTY layout | `crates/suggest` | `SuggestRules`, `PreviousWords`, `AutoCorrectionUndo`, `TapOffset`, `TapTrail` and `QwertyGeometry`; obsolete case/character/correction facades are test-only (package 20) | `SuggestRulesMatchTest` against `ReferenceSuggest`: every code point and every UTF-16 unit through each rule, 200,000 random words and corrections, 200,000 taps and 20,000 tap trails. The same on the emulator: none of 15,586,581 differed. Typing the same keys on both builds gave the same text |
| The words typing learns and keeps: words and word pairs the user has typed, words whose correction they undid, and their personal dictionary's words and shortcuts. Kotlin still reads and writes the files; Rust parses and writes their text | `crates/suggest` | `LearnedWords`, `KeptWords` and `PersonalWords`, now wrappers | `WordStoresMatchTest` against `ReferenceStores`: 2,000 random runs of learning, undoing, saving and loading damaged files on each kind of store, asking both the same questions after every step, and 100,000 random personal dictionaries. The same on the emulator: none of 5,647,957 differed. Typing the same keys on both builds gave the same text and saved the same files, before and after a restart |
| Swipe typing's word list and ranking: which words a swipe can spell, how strongly the glide classifier favours each, and how the words before a swipe re-rank its guesses. Native context and ranking share one exchange (package 20); Kotlin holds Android models and ownership | `crates/suggest` | `GlideRanker` and `SwipeWords`' rules, now wrappers | `SwipeRulesMatchTest` against `ReferenceSwipe`: every bonus for pair counts up to a million and for every n-gram and probability from -260 to 520, 100,000 random rankings (some longer than 20 guesses, where a sort that isn't stable would show), every frequency, every UTF-16 unit and 100,000 random words, and 20,000 random dictionaries with the user's words. The same on the emulator: none of 5,798,258 differed, frequencies to the bit. Swiping the same words on both builds, with and without a typing habit and a learned word, gave the same text. Building the 160,000-word list takes 12-15 ms, against Kotlin's 14-17 |
| The suggester and what typing learns: which corrections, completions and the user's own words the strip shows and in what order, whether to auto-correct, the spell checker, next-word predictions, and which spelling of each finished word is learned and when it is taken back. A1 owns the Rust decoder/session; package 20 learns directly through that session without a Kotlin callback | `crates/suggest` | `LatinSuggester` and `WordLearner` own Android IO/models and native lifetimes; legacy callback adapters are host/fixture-only | `LatinRulesMatchTest` against `ReferenceLatin`: 40,000 random strips and spell checks over a made-up dictionary whose every answer is a pure function of the question, with random personal and learned words, thresholds and taps, and 1,500 random runs of finished, swiped and undone words on two learners, comparing what each stored. It fails for each of 60 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 246,648 differed; and over the bundled dictionary, none of 4,000 typing states once both saw the same answers from it (see below). Typing the same keys on both builds (corrections and their undo, "came fro", a made-up word learned, suggestions and predictions tapped, a swipe) gave the same text and saved the same files, before and after a restart. Those historical pre-A1 measurements were 7.4 ms, dominated by the then C++ decoder; current native-session proof is in [A1](stage2-a1.md) and [package 20](stage2-consolidation.md) |
| Consolidated engine exchanges: direct native learning, one native glide-context/rank call, packed UTF-16 reader text, and production facades removed. Android node/PCM callbacks remain | `crates/suggest`, `crates/reader`, `crates/awnative` | `NativeLearnerNative.call`, `NativeGlide.rank`, `ReaderRules`; Android IO/lifetime hosts stay | [Package 20 inventory and proof](stage2-consolidation.md): 30,000 learner events, 10,000 real-dictionary glide cohorts, retained reader comparisons and all 65,536 UTF-16 units, restored mutations, paired APK timing and both-renderer device checks. Source external declarations 358 → 360; normal Android callback sites 8 → 5, with reference-only APIs retained |
| Reading a page aloud: which block reading starts at, how the pieces a walk of the screen found join into paragraphs, where a selection starts, where reading carries on after a scroll, how text is cut into what the speech engine takes, whether the words being read still fit in view, and the speaker key's tap and hold. Kotlin supplies Android node/playback operations; package 12 owns the C generator, queues and full selection/order | `crates/reader` | `PageText` and `SpeakerKey` keep Android segmentation/reveal/clock ownership; package 20 packs bulk text, and FollowAlong.fits is test-only | `ReaderRulesMatchTest` against `ReferenceReader`: 40,000 random pages, joins and scrolls, 20,000 texts to speak (some about as long as the speech engine takes), 100,000 views and 2,000 runs of presses on the speaker key. It fails for each of 63 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 400,000 differed. Reading a Settings screen aloud on the emulator found its 23 blocks, read and stopped. Joining and finding take about two thirds and half of Kotlin's time; carrying on after a scroll takes 23 µs against Kotlin's 0.3, the cost of handing Rust the page's text, once a scroll |
| The screen walk's decisions: whether a scrollable view goes sideways (a carousel or a pager), how much of a list is in sight, which of the screen's lists is the one that scrolls and which text is inside it, and how following along brings a web page's piece into view. Kotlin fetches Android nodes and executes scrolling; package 12 owns the full traversal policy | `crates/reader` | `ScreenWalk` and `FollowAlong` call `ScreenWalkRules` directly; ScreenRules/Box are test-only after package 20 | `ScreenRulesMatchTest` against `ReferenceScreen`: 40,000 random scrollable views and class names, 100,000 pairs of boxes (some as wide as `Int` goes, so their width wraps as Kotlin's does), 40,000 random trees of scrollers holding text and 100,000 pieces to bring into view. It fails for each of 51 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 620,001 differed. Reading a Settings screen aloud on the emulator walked the same 41 nodes into the same 23 blocks, 22 in the list, read and stopped. Placing 600 pieces in the list takes 3 µs against Kotlin's 6; the rest take about the same, under 2 µs a call |
| The GIF panel's rules: editing the search box and offering the word just typed, the order of the category chips, which keys and swipes reach the search box, the problem shown when GIPHY fails, which searches the result cache keeps and for how long, and the order of recently sent GIFs, saved and loaded. Kotlin still calls GIPHY, holds the GIFs and draws the panel | `crates/gif` | `GifSearch` calls text rules directly; SearchText is test-only after package 20. GifSearch.chips/problem, GifSearchKeys, GifResultCache and GifRecents keep UI/IO seams | `GifRulesMatchTest` against `ReferenceGif`: 100,000 random queries and texts (odd spaces, emoji and half emoji, apostrophes), 40,000 words for the chips, every key code from -10,000 to 13,000 with every key type, 40,000 swipes, every status near HTTP's, and 2,000 random runs each of a result cache (comparing what it holds as well as what it answers) and of recents. It fails for each of 63 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 1,086,149 differed. Every rule takes a few µs at most; sending a GIF again takes 4.5 µs against Kotlin's 1.2 |
| Reading GIPHY's answers and building its URLs: the JSON read exactly as kotlinx.serialization 1.12 read it into the panel's classes (members without commas, only four whitespace characters, unknown values skipped by brackets alone, raw control characters in strings), sizes read as Kotlin's `toFloatOrNull` and Java's parser read them, to the bit, which GIFs are kept, and each search's URL encoded as Android's `URLEncoder` encodes it. Kotlin still makes the call | `crates/gif` (`json`, `java_float`, `giphy`) | `Giphy.parse` and `Giphy`'s URLs, now a wrapper | `GiphyRulesMatchTest` against `ReferenceGiphy`: 60,000 random GIPHY-like answers, a third of them broken (commas, whitespace, escapes, nulls, repeats, cuts), 100,000 sizes (decimals and hexadecimals exactly halfway between two floats, far exponents, suffixes, padding) and 30,000 URLs for odd searches. It fails for each of 55 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 350,000 differed. Desktop Java 22 and later lose half an emoji at the end of one of `URLEncoder`'s 8-character chunks, where Android and Java 21 write `?` as the Rust does; the JVM test allows exactly that and the emulator holds the Rust to Android's own encoder. Reading a 30-GIF answer takes 0.41 ms against Kotlin's 2.4; a search's URL 1.0 µs against 5.4. Trending, a search and sending a GIF work on the emulator with a real key |
| The text keyboard's geometry: each key's flex factors, how keys in a row grow and shrink to fill it, row margins, each key's touch and visible bounds (end keys' touches reach the edges), the desired key's size, and which key a touch lands on. Kotlin still turns dp into pixels and holds the keys | `crates/keys` | `TextKeyboard.layout` and `getKeyForPos`, `TextKey`'s flex tables, `TextKeyboardLayout`'s desired key, now wrappers | `TextKeyboardMatchTest` against `ReferenceTextKeyboard`: 40,000 flex lookups, 20,000 desired keys and 10,000 random keyboards laid out and touched on and one float either side of key edges. It fails for each of 40 small breaks. On the emulator none of 1,485,902 differed. Laying out a QWERTY keyboard takes 2.2 µs against Kotlin's 4.0; finding a touched key is the same, 0.2 µs; a key's flex 0.2 µs against 0.04, once per key per layout. Typing works on the emulator |
| Swipes and the start of a glide: whether a finger's move or lift is a swipe, its direction of eight and how many units it went (Java's `atan2` and `toDegrees` matched to the bit), and whether a press becomes glide typing, which points the glide gets, and when it completes or is cancelled. Kotlin still reads touch events and the velocity tracker and calls the listeners | `crates/gestures` | `SwipeGesture.Detector` and `GlideTypingGesture.Detector`, now wrappers | `GesturesMatchTest` against `ReferenceSwipeGesture` and `ReferenceGlideTypingGesture`: 3,000 runs of 60 swipe steps and 3,000 of 40 glide events, plus the one point found to sit on a direction's edge. It fails for each of 64 small breaks. On the emulator none of 759,568 differed. A swipe move takes 0.03 µs against Kotlin's 0.13; a glide move about the same, 0.3 µs. Gliding a word and swiping the space bar to move the cursor work on the emulator |
| The keyboard's own spaces: the phantom space after a picked word, the space put before or after punctuation (off by default), deleting the space before a mark, and the flags that remember both, kept or dropped as the cursor moves. Kotlin still cuts the text before the cursor by graphemes, and only when a rule asks for it | `crates/spacing` | `EditorInstance.commitChar`'s spacing, `PhantomSpaceState.determine`, the two flag classes, `PunctuationRule.Fallback`, `FlorisLocale.supportsAutoSpace`, now wrappers | `SpacingMatchTest` against `ReferenceSpacing`: every UTF-16 unit in each place the rules look, then 4,000 runs of 50 random keystrokes, commits and cursor moves, comparing answers, flags and which texts were looked up. It fails for each of 40 small breaks. On the emulator, where Java's character rules differ, none of 2,164,983 differed. A keystroke's spacing takes 0.4–2 µs against Kotlin's 0.03–0.2, the cost of crossing into Rust, far below a keystroke's time. A phantom space after a picked word works on the emulator |
| Long-press popups: how the extended popup's grid is laid out (rows, which side it grows to, where the prioritized keys sit, its bounds), which key the finger is on as it moves or that it left, and the order of a key's popup keys by hint mode, keeping the old code's habit of handing back the set's own list in a few cases. Kotlin still builds the keys and draws the popup | `crates/popup` | `PopupUiController.extend` and `propagateMotionEvent` (through a new `ExtendedPopup`), `PopupSet.initPopupList` and `mergeInternal`, now wrappers | `PopupMatchTest` against `ReferencePopupGrid` and `ReferencePopupSet`: 100,000 random layouts (NaN, infinite, negative and huge floats, Int extremes, keyboards narrower than the popup), 200,000 finger positions either side of each edge, 20,000 lookups and 6,000 runs on popup sets. It fails for each of 40 small breaks. On the emulator none of 618,254 differed. A finger move takes 0.03 µs either way; laying out a popup 0.7 µs against 0.1, and ordering one 1 µs against 0.3, once per long press. Long-pressing e showed its accents around the 3 and sliding picked é on the emulator |
| The shift key and capitals: shift's press, release, caps lock and cancel under both capitalisation behaviours, automatic capitals, unshifting after a typed key, the double-tap timing, and the double-space full stop. Kotlin still knows which keys are down, reads the clock and settings, and does `fixCase`, which needs locale casing (Turkish, Lithuanian) the character tables don't have | `crates/shift` | `KeyboardManager`'s shift handling and double-space check, `InputEventDispatcher.isConsecutiveDown`/`Up`, through a new `ShiftLogic` | `ShiftMatchTest` against `ReferenceShift`: every state, event, behaviour, caps mode and flag combination, 5,000 random runs of 40 events, 200,000 double taps around the timeout with wrapping clocks, and every UTF-16 unit around 26 partner characters for the double space. It fails for each of 50 small breaks. On the emulator, where Android's regex `\s` takes more spaces than the JVM's, none of 3,920,160 differed. Each call takes 0.04 µs against Kotlin's 0.01; the double-space check 0.5 µs against 1.2. Shift, caps lock and the double-space full stop work on the emulator |
| Generic rule-table composing, including Vietnamese Telex: what a keystroke deletes and types given the text before it | `crates/composing` | `Composer.WithRules.getActions`, through `ComposingRules` | `ComposingMatchTest` against `ReferenceWithRules`: the bundled Telex rules, random rule tables, case changes, lone surrogates and every UTF-16 unit. Hangul, kana and their feature-only tests were removed from the Latin-only keyboard |
| Emoji: reading the bundled emoji files (categories, skin-tone sets, the by-tone lists), an emoji's skin tone and hair style, which locale's file to load, and the `:` suggestions, scored, sorted and cut to size. Kotlin still reads the files and settings and builds the emoji it shows; the parsed file stays in Rust so each keystroke is one call | `crates/emoji` | `EmojiData`'s parsing, `Emoji`'s tone and hair style, `EmojiSuggestionProvider`'s query check and scoring, now wrappers | `EmojiMatchTest` against `ReferenceEmoji`: every bundled file, 2,000 damaged copies of them, and 20,000 random queries, tones and locales each. It fails for each of 40 small breaks. On the emulator none of 163,808 differed (17,463 file, 58,883 damaged-file and 87,462 query checks). Reading a file takes 3.5 ms against 11; a `:` keystroke's suggestions about 0.75 ms against 0.8, still off the main thread. Typing `:smi` offered ☺️ 😏 😼 on the emulator |

Many large pure-logic pieces are in Rust. The next stages move the remaining decisions, then
the keyboard drawing and settings models, so Kotlin becomes a small Android shell.

### Stage 2.1: key/domain boundary and proof (2026-11-12)

The existing `keys` crate now owns domain decisions as well as geometry. Codes have one
canonical Rust table; Kotlin exposes named values from it. Predefined keys carry their types
and labels in Rust. Kotlin retains the kotlinx.serialization classes and enums, Compose's
layout-direction value, observable state and batch notifications, Android fact gathering,
ICU's mark predicate and locale casing/cache, and assembly of key/popup objects. Rust chooses
children and popup sources by index, so JNI does not need to understand Kotlin objects or
call back into an evaluator. Clipboard queries still stay behind the locked-device guard.
Layout assembly/cache and Android labels/icons remain for their later work packages.

The JVM differential tests retain the replaced Kotlin as `ReferenceKeySelectors`,
`ReferenceKeyboardState`, `ReferenceKeyMetadata`, `ReferenceKeyCodes` and `ReferenceKeyRules`:

- 93 bundled layouts across 132 JSON files, 5,840 key nodes in 80 shift/variation/direction/width
  combinations: 467,200 cases. Both sides receive the same casing snapshots on the JVM,
  because Android ICU itself was not ported. Another 40,000 seeded nested selectors include
  missing children and children whose computation returns null; only missing children fall back.
- All 65,536 low state registers and 20,000 random full-width registers, then 2,000 seeded
  runs of 40 public mutations (80,000 transitions), including snapshots and wrapping/high bits.
- 123,003 key-code inputs through lookup, space/currency classification and UTF-16 string
  rules; all 60 predefined key records and every named code pinned against the old values.
  Invalid code points are ignored, while lone surrogate code points are retained, as before.
- 200,300 currency lookups over 100 seeded sets of different lengths, plus all 26 bundled
  currency sets (52,026 lookups) and 80 explicit space/type cases.
- All 5,120 popup-source masks/variations and 598,026 enabled/visible code/fact combinations,
  plus popup modes/groups and phone hints. Fallback, disabled utilities and password cases
  retain the old precedence.

The deliberate mutation made an unshifted case selector choose its upper child. Both the
bundled-layout and seeded-selector tests failed; the change was removed and the full checks
passed. Tests also caught and fixed a port mistake: code-info labels for positive control
characters use appendCodePoint, whereas display labels use the key's label. A separate audit
against the original Kotlin caught a temporary extraction regex missing the digit in
VIEW_SYMBOLS2: correcting the reference first made the lookup test fail, then fixing the
Rust restored parity. Existing quirks
stay: currency lookup takes abs(code), including positive codes and Int wrapping, and
SHOW_SUBTYPE_PICKER is predefined but absent from the code-info lookup.

Using the plan's non-blank tracked-main-source measure, stage 2.1 removes 487 Kotlin lines
and adds 991 Rust lines (including JNI); Java is unchanged. Totals are 53,321 Kotlin,
26,297 Rust and 627 Java, about 32.8% Rust. This is a migration count, not a claim that the
90% target has been reached. Test references do not count toward either total.

On the muted s24_like emulator (`local`, `emulator-5554`), the baseline and port builds were
installed with `scripts/install-dev.sh` and driven through the T3 device session. The same
key sequence matched in alphabet/symbol and numeric fields, including automatic capitals,
manual shift, caps lock and release back to lowercase. Long-press e showed the same accent
and number-hint grid; the dollar key and its five currency alternatives matched, including
the prioritized cent key. Password input stayed masked, without suggestions or an enabled
mic. Phone keys retained their secondary labels. The requested FATAL/AndroidRuntime logcat
search found only routine runtime startup/exit messages, with no fatal or error-level runtime
entries. The temporary field-check APK is a build artifact, not a new app feature.

### Stage 2.2: layout assembly/cache boundary and proof (2026-11-12)

The existing `keys` crate owns the layout load plan for each mode and number-row setting,
the modifier type for an explicit main-layout override, arrangement merging, first-row number
hints and bottom-aligned symbol hints, and computed-keyboard cache key packing/invalidation.
Keys cross this boundary as indices with row sizes and plain-key codes, not serialized objects.
This keeps selectors, labels, popup objects and repeated insertion markers intact, including
empty rows and the difference between an unavailable layout and a valid empty layout.

Kotlin retains asset/archive reads, kotlinx.serialization decoding, metadata/resource lookup,
coroutines and their deferred file-read caches, warnings, TextKey allocation, Android locale
casing/cache and display resources. The already-ported key and popup rules remain in Rust.
`LayoutCache` holds managed keyboard objects under Rust's keys; the original subtype data-class
hash is an opaque input, preserving the old hash-collision behaviour until the subtype package.
There are no JNI callbacks or native handles for keyboards, so no new lifetime/locking protocol.

`LayoutAssemblyMatchTest` retains the replaced decisions and loops in test-only
`ReferenceLayoutAssembly` and the old cache policy in `ReferenceTextKeyboardCache`. It checks:

- All 93 bundled layout files (94 indexed entries), all 46 retained subtype presets and all
  12 keyboard modes, with the number row off and on: 103,776 complete keyboards and 1,581,429
  evaluated keys. An assertion proves the index covers every bundled layout file. Each layout
  is substituted in its own slot, including modifier and numeric-row layouts. Main metadata's
  explicit modifiers are resolved too.
- Complete ordered key lists: codes, types, labels, groups, multi-code-point data, popup and
  hint keys/order under seeded hint configurations, flex factors, row/column positions and
  touch/visible bounds to the Float bit. The reference uses the old Kotlin geometry; production
  uses Rust geometry. Both receive identical JVM locale-casing snapshots because Android ICU
  was not ported. Android resource labels/icons remain unchanged outside this pure boundary.
- Another 4,000 seeded keyboards (57,034 evaluated keys) with missing/empty extension and layout
  references, number/hint settings and shift/variation/direction/width states; also 4,000 synthetic
  arrangement sets with unavailable/empty layouts, empty rows, repeated insertion markers,
  selectors that produce code 0, and selectors that produce no key.
- 40,000 cache operations followed by 2,880,000 lookups against the original per-mode Int-hash
  cache: overwrites/collisions, every clear scope, negative hashes and Int boundaries.

The deliberate Rust mutation changed the plain-key insertion marker from code 0 to code 1.
Both the bundled-layout matrix and seeded-layout test failed at arrangement comparison; the
cache test still passed. Restoring code 0 restored the suite. The first matrix run also caught
an invalid test fixture supplying a modifier field that SubtypeLayoutMap does not have; the
fixture now substitutes those layouts at loading, retaining coverage of every modifier.
There are no intentional behaviour changes.

The full command `./gradlew --stacktrace test :app:lintDebug :lib:rust:check
:lib:rust:oracleCheck assembleDebug` exited 0 for both the rules/reference commit and production
routing. Lint reports zero errors and exactly the three known warnings
(AndroidGradlePluginVersion, NewerVersionAvailable, ChromeOsAbiSupport).

On the muted s24_like emulator (`local`, `emulator-5554`), both builds were installed through
`scripts/install-dev.sh` and driven with the T3 device session. The same steps checked English
QWERTY, Croatian QWERTZ, Catalan with its extra ç key and long-press e accent, both symbol pages,
number row off/on, portrait/landscape and a height change from 100% to 125% and back. Seven
paired screenshots' key regions are pixel-identical: QWERTY, QWERTZ, Catalan, both symbol pages,
125% height and the reopened landscape keyboard. The first rotation screenshots captured the
settings field losing focus on both builds; a rebuilt baseline with the original LayoutManager
and cache confirmed the actual reopened landscape keyboard too. Unchanged editor state controls
the enter/Done icon; matching field/state snapshots match it. Long-press e commits the same
accent on both builds, and deletion removes it. The requested logcat search found only routine
debug/info AndroidRuntime startup/normal-exit messages, with no FATAL or error-level runtime
entries. The three temporary subtypes were removed, number row/utility key/height restored,
and AudioWhisper was restored as the selected IME. Screenshots and APKs are local QA artifacts,
not shipped code.

The tracked-main-source recount removes 151 non-blank Kotlin lines and adds 273 Rust lines
(including JNI), a net addition of 122 lines; Java is unchanged. Totals: 53,170 Kotlin,
26,570 Rust and 627 Java, 33.06% Rust (previously 32.77%). Test references are excluded by the
90% section's rules. Kotlin shrinks; this package does not claim the overall target is reached.

### Stage 2.3: Snygg grammar boundary and proof (2026-11-12)

The new Java-free `snygg` crate owns element/annotation parsing, attribute ranges and edits,
selector parsing, exact comparison results, canonical rule formatting, schema/version policy
and error messages, ordered document plans and serialized root keys. Models use UTF-16, so
quoted values and even lone surrogates survive JNI. `awnative` only packs/unpacks these models;
there are no callbacks, locks or native handles. The renderer can reuse `grammar::Rule`,
`attributes::Attributes`, and `document::{schema, plan, serialized_keys}` without Java.

Kotlin keeps the public Snygg class names, constructors and signatures, managed object allocation,
the kotlinx JSON syntax codec and pretty printer, and property/value decoding. Keeping that codec
preserves its exact malformed-JSON/root-shape/String-decoding exceptions. Rust receives the
decoded ordered keys and schema, returning source indices and first-insertion slots: equal rules
replace the value but retain the first key and position. Rule errors are deferred in the plan,
so an earlier Kotlin property/value error still wins. The public compile-time `SCHEMA_V2` constant
stays for source compatibility and is checked against the reference/native metadata.
`SnyggSpecDecl`'s encoder/schema DSL stays in Kotlin; its `RuleDecl.pattern` values now come from
Rust. The unchanged `SnyggJsonSchemaGenerator` receives byte-identical regex patterns.

At this boundary, package 4 still owned property sets, value syntax/encoders, dimensions, colours,
fonts, padding, shapes and variables, including required/invalid-property/value errors. Package 5 still owns
compilation, matching, inheritance/cascade, editor operations and schema-generation behaviour.
Unknown element and attribute names remain syntactically accepted. Numeric attribute strings
still sort lexically before range coalescing, descending ranges still become empty lists, and
the original schema regex's unescaped dots still match a Unicode code point. There are no
intentional behaviour changes.

The two JVM match suites retain the replaced Kotlin in `ReferenceSnyggRule`,
`ReferenceSnyggAttributes`, and `ReferenceSnyggStylesheet` (including independent grammar error
messages). They compare parsed models, map/list order, exact comparator integers, serialization,
round trips and exception classes/messages, covering:

- All 12 bundled stylesheet files and all 28 existing inline stylesheet test documents.
  Discovery scans tracked JSON and extension archives across the repository, not a fixed asset
  list. Every file runs with all 64 configuration flag combinations; inline fixtures use strict
  and fully lenient configurations. No other extension/test stylesheet files currently exist.
- 12,000 seeded rules, 20,000 seeded comparison pairs, 4,000 attribute-edit histories with
  40,000 operations, and every UTF-16 unit in font names and quoted attribute values (131,072
  rule cases), including lone surrogates and the platform-specific regex whitespace table.
- 4,000 seeded documents and 1,200 mutations of the real stylesheets, plus directed schema,
  version, malformed JSON, unknown names, bad selectors, duplicate raw/canonical rules and
  property-versus-grammar error-precedence cases under every configuration flag combination.

The deliberate mutation reversed Rust's selector comparison. The JVM grammar match test failed
on `key:disabled` versus `key:hover` (expected 1, got -1); restoring it restored the suite.
A temporary Android-only reference probe checked the full 131,072 character-sensitive cases
on `aw_keyboard` with zero mismatches, logging counts only. The probe and reference sources
were removed from production before the final build.

The final full command `./gradlew --stacktrace test :app:lintDebug :lib:rust:check
:lib:rust:oracleCheck assembleDebug` exited 0. Lint has zero errors and only
AndroidGradlePluginVersion, NewerVersionAvailable and ChromeOsAbiSupport warnings. A separate
`:lib:snygg:generateJsonSchema` build exited 0 and reproduced the tracked schema byte for byte.

Before and after builds were installed only through `scripts/install-dev.sh` on the muted
`aw_keyboard` emulator (`local`, `emulator-5556`), with the keyboard enabled/selected and the
Rust renderer prototype off. Captures cover Floris Day, Floris Night and Floris Pure Night in
the same empty preview field, plus the editor on an unmodified temporary copy of bundled Floris
Day (bundled extensions are read-only). All three keyboard regions and the editor are
pixel-identical. Whole-screen PNGs differ in the system clock; the Pure Night pair also differs
in 239 blinking-caret pixels outside the keyboard. The requested logcat search is empty.
Screenshots live under ignored `build/stage23/screenshots/{before,after}-*.png`, not in git.
The temporary extension was removed, original Follow system theme mode restored, and the
emulator shut down after testing.

Using the plan's non-blank tracked-main-source measure, this package removes 197 net Kotlin
lines and adds 790 Rust lines including JNI. Totals are 53,385 Kotlin, 29,171 Rust and 627 Java:
35.07% Rust, up from 34.36% at this checkout's starting commit. Tests and temporary probes are
excluded; the public Kotlin API documentation remains in place.

### Stage 2.4: Snygg value boundary and proof (2026-11-12)

The Java-free `snygg` crate now owns all 28 built-in value encoders: dimensions and percentages,
sRGB colours and Material You references, font families/style/weight, padding, corner shapes,
text/appearance enums, maximum lines, flex URIs, variables and the inherit/undefined/yes/no
keywords. `value::Value`, `Unit`, `ShapeKind` and `Corners` hold plain numbers, enums and UTF-16
strings for the future renderer/resolver. Rust supplies specs, alternatives, defaults, parsing,
validation, canonical serialization, first-success encoder selection and ordered property
validation. `awnative/src/snygg_value.rs` only transfers UTF-16 strings and f32 bits; no Java
references, callbacks or native handles enter the crate.

Kotlin keeps public value names, constructors and method signatures, platform-object conversion
and the kotlinx JSON codec. Property inputs carry deferred JSON string-decoding failures into
Rust's ordered plan, preserving which invalid-property/value/JSON error wins and when required
properties are checked. Kotlin reconstructs the original public exceptions. Missing variable
names remain references; resolving them belongs to package 5. The renderer prototype is untouched.

The public `SnyggValueSpec` builder and generic enum encoder still accept caller-defined JVM
regexes and conversion functions; those custom implementations retain their JVM defaults.
Every built-in spec parses and packs in Rust. Built-in keyword specs retain the public
`SnyggKeywordValueSpec` runtime type with a native binding, because the unchanged schema
generator distinguishes keyword constants/enums by that type. The old `@CallSuper` requirement
was removed because a native implementation replaces the default parser/packer completely.
The schema DSL/generator, cascade, inheritance, matching and editor model remain package 5.

The original value implementations live only under `src/test/.../reference/Reference*.kt`.
`ReferenceSnyggPropertyCodec` retains the original property loop. Field-only bridges compare
reference results with public production objects without invoking the production parser.
The JVM suites compare fields, defaults, spec metadata/captures/packing, canonical output,
round trips, encoder choice and exception classes/messages, covering:

- All 12 tracked bundled stylesheets, discovered repository/archive extension sheets and all
  28 inline stylesheet fixtures: 40 documents, 3,254 value occurrences across all 28 encoders.
  The existing stylesheet suite also compares all 64 configuration combinations.
- 7,000 seeded malformed/mutated inputs across every encoder, directed whitespace/case/unit,
  numeric, colour, keyword, variable, font, padding, corner and URI edge cases, plus 4,000
  seeded property sets and directed required/invalid-property/value/JSON precedence cases.
- Every UTF-16 unit in font names and URI paths (131,072 cases), lone/paired surrogates,
  platform regex whitespace, every dynamic colour name, 5,000 generated colour/size/padding/
  corner groups, all alpha-byte values, and URI authority/IPv6/query/escape/fragment cases.
- 20,000 seeded arbitrary f32 bit patterns plus directed bounds, signed zero, NaNs and
  infinities through four numeric serializers; invalid programmatic enum/text/colour/padding/
  corner values and every wrong encoder/value combination.

No grammar changes are intentional. In particular, cut-corner dp values still parse their
components as integers, small sp values and exponent notation remain rejected by their
patterns, rgba's alpha grammar still rejects `1.00`, and weight aliases serialize to names.
The deliberate mutation changed percentage decoding from division by 100 to division by 99.
`SnyggValueMatchTest` failed on `100%` (expected `1.0`, got `1.010101`), and Gradle exited 1.
The mutation was restored before validation.

The final full command `./gradlew --stacktrace test :app:lintDebug :lib:rust:check
:lib:rust:oracleCheck assembleDebug` exited 0 (Gradle's own exit status, no output pipeline).
Lint reports zero errors and only AndroidGradlePluginVersion, NewerVersionAvailable and
ChromeOsAbiSupport warnings. `:lib:snygg:generateJsonSchema` also exited 0 and reproduced
`schemas/stylesheet.schema.json` byte for byte. All changed source files are at most 500 lines;
no warning suppressions, lint baseline or ignored/deleted tests were added.

Emulator acceptance is **not complete**. `device_open(local, aw_keyboard)` returned runtime id
`emulator-5556`; all commands were bound to it, and `emulator-5554` was never touched. Streams
1/2/3/5 were muted and alarm stream 4 set to its minimum 1; Do Not Disturb was never enabled.
During baseline setup, the Android snapshot helper failed with `Android snapshot helper returned
insufficient foreground app content`. Its advised retry failed identically (diagnostic
`mwej0vkr-111dea17`). The screenshot shows the system IME picker; the captured
`build/stage2.4/screenshots/before-floris-day.png` is a diagnostic, not a valid keyboard baseline.
Device work was stopped and `device_close(local, emulator-5556, shutdown=true)` completed,
following the requested fallback. No changed APK was installed. There are no valid screenshot
pairs, dynamic-theme/editor captures, colour/size edit-save-reopen checks or final runtime
logcat check for this package. Theme mode had been changed from Follow system to Always day
before the failure and could not be restored after device work stopped. The Rust renderer
setting was never toggled. Android-specific differential and visual acceptance remain open.

Using the plan's non-blank tracked-main-source count, this package removes 317 net Kotlin lines
and adds 1,574 Rust lines including JNI; Java is unchanged. Totals: 53,068 Kotlin, 30,745 Rust,
627 Java (36.41% Rust versus 35.07% at `93876574`). Tests and generated tables are excluded.

### Stage 2.5: Snygg compiler, resolution and editor boundary (2026-11-12)

The Java-free `snygg` crate owns stable per-element cascade ordering, selector/attribute
matching, single-pass variable substitution (including missing variables and unresolved
aliases/cycles), explicit/implicit inheritance, undefined/unset handling, font scaling and
named light/dark palette resolution. Built-in property declarations and inheritance flags
live in `spec`. `editor` supplies ordered property edits, rule add/rename/conflict decisions,
remove/copy snapshots and the editor's known-element display order. `awnative` only transfers
UTF-16 data and opaque managed-object tokens; there are no callbacks or native handles.

For milestone B, use `resolve::CompiledTheme<Value>::from_json(stylesheet_json)` and
`resolve(&Query { element, attributes, selector }, parent_properties, font_multiplier)`.
`resolve_with_palette` also takes light/dark property maps of named plain colour values.
The compiled model retains font source sets for the renderer to load. `Properties`,
`PropertySets`, `Stylesheet`, `Query`, and `CompiledTheme` are ordinary Rust data; no JVM is
needed. The standalone JSON entry point uses ordered serde JSON for valid v2 documents;
Kotlin continues using its existing kotlinx codec and exact diagnostics. The `render` crate
and its existing managed handoff are unchanged.

Kotlin retains the public models/signatures, map/list identities and allocation, typed
Compose factories/conversion (including wide-gamut colours, unspecified and em TextUnits),
Material You palette materialization, Android asset/font loading and the `ui/` composables.
The editor's public mutable property map uses native slot plans while preserving insertion
order, live entries and iterator removal. The public TreeMap/custom comparator API remains;
the application's named-element comparator and rename/add decisions now call Rust. Android
workspace/file operations and extension metadata validation remain Android/app concerns.

The generic `SnyggSpecDecl` DSL and `SnyggJsonSchemaGenerator` stay in Kotlin: they accept
caller-supplied JVM regexes, encoder/spec objects and builder callbacks, and the generator
uses runtime class-derived definition IDs plus the existing kotlinx pretty writer. Moving
that extensibility/codec here would duplicate the allowed JVM boundary. The built-in DSL
is now an adapter over Rust declarations. A separate `:lib:snygg:generateJsonSchema` run
exited 0 and `cmp` confirmed byte-identical tracked schema output.

Test-only `ReferenceSnyggTheme`, `ReferenceSnyggPropertySetEditor`,
`ReferenceSnyggStylesheetEditor` and `ReferenceSnyggSpec` retain the replaced implementations.
The compiler/property references use ordinary managed maps and frozen inheritance policy.
The differential suites cover:

- Every one of the 12 bundled stylesheets, all 82 application elements, all 12 keyboard
  modes, four shift states and five selectors. There are 868,680 direct queries, including
  every concrete app attribute-domain value and complete Cartesian products of each rule's
  attribute predicates (including absent/unmatched classes). Values in a predicate class
  have identical membership in every selector rule; root/window inherited contexts are
  compared for every mode/shift/selector combination.
- 1,000 seeded programmatic stylesheets and 30,000 queries/parent transitions, including
  missing/cyclic/one-level variables, empty attributes, duplicate attribute values, undefined,
  inherit, all built-in value defaults, non-sRGB colours, em/unspecified units and non-finite
  or signed-zero font multipliers. The tests caught and fixed an em-to-sp transport bug.
- 120 seeded editor histories, 7,200 operations: add/rename/conflict/missing/no-op, remove,
  independent copy, set/unset, multi-font-set edits, iterator removal and reverse reinsertion.
  Every step compares ordered stylesheets, compact/pretty serialization, reloads, and exact
  validation exception classes/messages. Display ordering is compared independently too.
- The native integration test loads the actual bundled light/dark JSON and compares key,
  pressed-key, candidate, popup, focus, window and hint styles to a frozen oracle exported by
  the Kotlin reference along the current prototype query path. The JVM test also verifies
  this fixture. All 12 bundled files decode and resolve directly in Rust.

The deliberate mutation traversed compiled rules in reverse during resolution. The bundled
JVM match test failed on a pressed key's background and Gradle exited 1. Restoring forward
traversal restored the suite. The final command `./gradlew --stacktrace test :app:lintDebug
:lib:rust:check :lib:rust:oracleCheck assembleDebug` exited **0**. Lint has zero errors and
exactly AndroidGradlePluginVersion, NewerVersionAvailable and ChromeOsAbiSupport warnings.

On `aw_keyboard` (`local`, runtime `emulator-5556`), the before/after Floris Day, Floris Night,
Floris Day Borderless and Floris Day Material You keyboard regions are pixel-identical,
as is the unmodified Day-copy editor region. All pairs use Always day and system night mode
on; the Rust renderer prototype stays off. Keyboard comparisons cover `(0,1563)-(1080,2288)`;
the editor comparison covers `(0,300)-(1080,2170)`. Clock/caret/navigation differences are
outside these regions. Pressed/shifted captures were not added; those states are covered by
the resolver tests. Screenshots are ignored `build/stage25/screenshots/{before,after}-{day,
night,borderless,material,editor}.png`; comparison code/results are in the same ignored
`build/stage25` directory.

The Day stylesheet was copied into a temporary custom extension. Through the actual theme
editor, `key[code=97]` was added with `rgba(242,10,10,1)` background and a 24sp font override.
The preview changed colour and size (403 glyph-region pixels changed for the size alone).
Saving and reopening preserved both properties in the extension archive and produced a
pixel-identical keyboard preview. The temporary extension was deleted through the UI;
Always day + Floris Day was restored. `adb logcat -d` contained no FATAL or AndroidRuntime
matches. The runtime device was closed with shutdown enabled. Only `scripts/install-dev.sh`
installed the build. A stale-ref helper rejection recovered after the advised fresh snapshot.

The planning source measure removes 671 and adds 677 non-blank main Kotlin lines, **net +6**
(the API/transport/collection facade and splitting the editor UI into files offset the deleted
logic). It adds **794 Rust lines**, including JNI and the native JSON consumer. Totals are
53,074 Kotlin, 31,539 Rust and 627 Java: **37.00% Rust**, from 36.41% at `9f917ec1`.
No tests/generated tables are counted. New/changed source files stay at or below 500 lines.

### Sharing a core with the Mac app

The Mac app's text rules are Rust too, in [AudioWhisper-Core](https://github.com/jtn0123/AudioWhisper-Core)
(public), which the Mac app builds as a submodule:

- `transcript`: spoken punctuation, filler words, custom vocabulary, fitting a phrase to the text before
  the cursor. Merged into the Mac app in AudioWhisper PR #59.
- `apple-text`: the Swift and Foundation text semantics and ICU regular expressions those rules need.
- `correction`: the safety gate that rejects a model's "correction" that changed too much (PR #60).

Each was ported against the Mac's Swift with a differential test and a mutation check, as here. The
keyboard takes `transcript` as a git dependency pinned to a commit (`crates/awnative/Cargo.toml`) for its
filler words and vocabulary; bump the `rev` to take a newer core. The keyboard keeps its own spoken
commands, joining and capitals: they were written for phrases transcribed one at a time as the user
speaks, where the Mac's rules see a whole dictation at once, so neither set replaces the other.

That is where the Mac's port stops (2026-11-10). What is left of its Swift is Apple-framework glue
(AudioToolbox file reading, Accelerate's FFT, which beats plain Rust, SwiftUI and AppKit), the realtime
audio buffer, where a call into Rust on every audio callback would add risk for nothing, and prompt text
only the Mac uses. None of it is shared or faster in Rust.

### The LatinIME port

The C++ is ported against itself rather than a Kotlin copy. `crates/latin-oracle` builds
LatinIME's own sources for the computer running the tests (with the NDK's `jni.h` and a fake JNI
environment that runs the production `JNI_OnLoad`), so a test can put the same question to the
C++ and to the Rust: open the bundled dictionary, a word's probability, an n-gram's, its word
list, and a typing decode with taps and previous words. `cargo run --release -p latin-oracle
--example oracle -- gen 20000 20261109` writes seeded questions shaped like the keyboard's, and
`oracle run` answers them.

First, everything the keyboard never calls was deleted: the writable dictionary formats (v4 and
v402) and their writers, the dictionary-building and GC code, `BinaryDictionaryUtils`, and every
native method of `BinaryDictionary` but open, close, suggestions, the two probabilities and the
word list. The 20,000 questions got byte-identical answers before and after, and the comparison
caught a changed scoring constant (1,864 of the first 2,000 answers differed). What is left is
about 11,600 lines of code (16,800 with comments) in 138 files.

That remainder is now ported, in `crates/latin`, one module per area of the C++: `chars` and
`bytes` (character rules, integers and code points), `header` (the dictionary's header and its
attributes), `trie` (the version 2 trie: words, probabilities, word pairs, shortcuts and the word
list), `layout` (the keyboard and the taps on it) and `search` (the typing search, its node cache
and its scoring). The port keeps the C++'s types and order of operations, so its answers match to
the bit. Each module is tested against the C++ through the oracle, and each was broken on purpose,
one change at a time, to show the tests notice; the few changes they could not notice are ones no
input can tell apart.

The app runs on the port through `crates/suggest/src/session.rs`. One native session owns
its dictionary, traversal cache, QWERTY proximity model and retained output buffers. The Rust
suggester asks that session's `Dictionary` implementation directly: there is no Kotlin/Java
callback to decode, look up probabilities, or construct previous-word and geometry arrays.
Kotlin reads dictionary bytes and converts the returned UI models; JNI stays in `awnative`.
The old Java classes and Kotlin marshalling engine are test-only references, and the eleven
legacy JNI exports compile only for host tests. No LatinIME Java module or C++ library is built
or shipped in the application.

At stage 2 package 6, saved pre-change and new JVM/device replays match byte for byte:
4,000 seeded decoder requests (all retained slots, weights and probabilities) and 2,000 full
pipeline states (strip/autocorrect and spelling, with personal and learned words). A separate
4,000-question C++ oracle run also matches the native session. A reversed result-order mutation
fails the JVM match test. Both renderers retain the 300-press/eight-hold digests and match the
autocorrect/suggestion-pick fixtures. See [the session proof and reproduction](stage2-a1.md).

`lib/latinime/src/main/jni` stays unchanged as the reference `latin-oracle` compiles for tests.

### The typing scoreboard

`crates/scoreboard` measures how well the keyboard types held-out English, on this computer,
through the same Rust the phone runs: the native `Session` and `Suggester` (offensive words
blocked, the modest auto-correction threshold, eight candidates of which the strip shows three)
and the `glide` index with `glide_ranker`. What Kotlin still does around them (the glide index's
character table, `ln` of a swipe's weight, the "next" space, the point filter) is repeated as the
Kotlin does it. The keyboard is the shipped en-US QWERTY layout at a Galaxy S24 Ultra's portrait
size (1440 px at 3.75 px/dp), laid out by `crates/keys`. It is seeded: the same inputs give the
same table.

It reports next-word top-1 and top-3 after a space, keystroke savings (the word is tapped in the
strip as soon as it is shown, counting the tap), auto-correction on simulated taps (neutral and
left-thumb) through the tap-offset path, auto-correction for a typist who hits every key but never
types an apostrophe ("dont", "its" for "it's"), and swipe top-1 and top-3 on simulated paths.
Below the table it prints each typist's exact counts, for differences too small for its rounded
percentages. Each module's comment says exactly how it simulates; the results and their history
are in [language-model-results.md](language-model-results.md) and
[autocorrect-results.md](autocorrect-results.md). `--strength S` or `--threshold T` scores
another auto-correction strength.

The test text is held out from a public corpus, which is never committed, so get it first:

```bash
scripts/build-language-model.sh                        # download (pinned, checked), split, build, score
scripts/build-language-model.sh --run build/language-model/run-<time> score   # score again
scripts/build-language-model.sh --run build/language-model/run-<time> install # ship what was built
```

The script downloads into a new `build/language-model/run-<time>/` folder, checks each file's
SHA-256, splits it with `cargo run -p lexicon -- prepare` (one sentence in a hundred held out, by
a hash of its words), builds the dictionary and the n-gram model (below) and runs the scoreboard
on the first 5,000 held-out sentences of each source: for the AOSP dictionary, the one it built,
and that with the model. That takes about four minutes. To score another dictionary or model on
the same text:

```bash
cd lib/rust
cargo run --release -p scoreboard -- --dict path/to/main.dict [--model path/to/main.lm] \
    --test tatoeba=RUN/corpus/test-tatoeba.txt --test globalvoices=RUN/corpus/test-globalvoices.txt
```

`--limit N`, `--seed N` and `--threads N` change the sample, the taps and swipes, and the
parallelism; `--skip N` scores the sentences after the first N instead, to tune on text the
results don't use; `--typing W` and `--swipe L` change how much the session leans on the model
(`suggest::language_model::Tuning`). Neither crate ships: `awnative` doesn't depend on them.

Swipes are drawn by two simulators, each with a neutral hand and a left thumb: `trace`, a path by
rule, and `lognormal`, the sigma-lognormal model of human movement, so a change tuned to one
can't pass for free ([swipe-accuracy.md](swipe-accuracy.md)). They are decoded as the keyboard
does unless told otherwise: `--original` is FlorisBoard's classifier on the flat frequency curve,
and `--ends E`, `--dwell D`, `--reach R` (`glide::Tuning`), `--above A` (the curve's cost a step
above common) and `--boost B,M` (`swipe_words::Boost`) each change one thing from there.
`--history FROM` has the user type each test file's 5,000 sentences from FROM on first, into the
learned words; keep it clear of the scored ones. `--swipe-only` skips everything but swipes.

### Learning where the user taps

`crates/suggest/src/tap_aim.rs` is the model: per key and orientation, a running mean of the
user's tap offsets and their spread, with the limits that keep it safe (16 samples before a key
applies, one standard error and 0.03 key taken off, a 0.25-key cap, a 200-sample window, fading
for keys not used). `tap_learning.rs` decides what teaches it: words kept as typed, with no
backspace inside, confirmed by typing on and thrown away by a backspace, an undo or a replacing
suggestion. `crates/keys/src/aim.rs` is the hit-test rule both renderers share: a tap moves to a
neighbour only when both keys' aims agree. The NLP session's "aim" operation says whether a kept
word may teach (the setting, the field, incognito).

The app holds it in the same `TapTrail` handle as the tap positions (`awnative/src/suggest/tap_aim.rs`).
`LatinLanguageProvider` loads and saves `files/tap-aim.txt` (numbers only) and hands the decoder
aimed offsets. Compose's `TextKeyboard.getAimedKeyForPos` and the Rust renderer's
`Scene::aimed_key_at` (fed by `RenderNative.touch`'s trail handle and `aimed` flag) pick the key,
with the letter odds too (below). Why and the numbers: [decisions.md](decisions.md#learning-where-the-user-taps-2026-11-16),
[tap-aim-results.md](tap-aim-results.md).

The scoreboard measures it with a learning run: a simulated user types one long stream while the
keyboard learns from the words they keep, scored after a warm-up, with learning off, with (a)
alone and with (a) and (b):

```bash
cd lib/rust
cargo run --release -p scoreboard -- --model ../../app/src/main/assets/dicts/main_en-US.lm \
    --test tatoeba=RUN/corpus/test-tatoeba.txt --test globalvoices=RUN/corpus/test-globalvoices.txt \
    --limit 5000 --learning [--warm-up 2000] [--from-corrected nothing|unchanged|rebased]
```

`--from-corrected` tries teaching from kept auto-corrections too (not what ships).
`crates/scoreboard/src/learning.rs` says how the users are simulated.

### Keys resized by the letter odds

`crates/keys/src/resize.rs` is the rule both hit tests share: near a border, the key whose letter
is likelier to come next takes up to 0.15 key of its neighbour's room (`Resize::SHIPPED`), after
the learned aim (`key_for`: the aim first, and the odds never undo it). The odds come from the
n-gram model: `ngram::Model::next_units` sums the probability of the words that continue what was
typed by their next letter, and `suggest::letter_odds` turns that into each key's prior.

The suggest JNI works the odds out each time it suggests and keeps the latest
(`awnative/src/suggest/letter_odds.rs`, the word being typed and 26 numbers, never logged). At key
down, both hit tests take them only while `TapTrail::at_word` says they still hold: the letters
typed end in their word and no other key is down, or the space bar was the key before a word's
first letter. A new field clears the trail and forgets the odds with it, so a field where the
suggester never runs (a password, one without suggestions) never leans on another field's odds.
Compose asks through `TapPositions.retarget`, the renderer through `lean` in
`RenderNative.touch`; both pass the trail handle and whether to use the learned aim, nothing
else. Why and the numbers: [decisions.md](decisions.md#keys-resized-by-the-letter-odds-2026-11-16),
[key-resize-results.md](key-resize-results.md).

The scoreboard measures it with `--resize ACROSS,DOWN,CAP[,FLOOR]` (`0.04,0.025,0.15,0.1` ships):
each word is typed twice with the same taps, through the plain hit test and the resized one, so
the report counts the words each gets right that the other doesn't, the rare words outside the
dictionary among them. It works with `--learning` too. `scripts/key-target-mutations.py` breaks
each rule and requires a test to fail.

### The typing counters

The phone counts, privately, how typing goes: integers only, never a word, a character or a
position. `crates/nlp/src/counters.rs` counts what the NLP session sees ("learn" and "pick") and
what Kotlin passes through its "tally" operation (`TypingCounters.kt`: a keystroke, a backspace,
an undo). Nothing is counted in incognito or in fields without word suggestions (passwords, for
one). The file is `files/typing-counters.txt` in the app's private storage, saved with the learned
words (about five seconds after a change) and left out of backups:

```
typing-counters 1
words 812
kept_as_typed 655
...
```

| Counter | Counts |
|---|---|
| `words` | Words entered by any means: kept as typed, corrected, picked or swiped. A word in one place counts once: finished again there (its space deleted and retyped, a letter fixed) or put back as typed by undoing its auto-correction. A swipe taken back (deleted, undone or replaced) leaves the count, and what replaces it counts instead |
| `kept_as_typed` | Words finished as typed, with no auto-correction |
| `autocorrected` | Auto-corrections committed (a space or punctuation replaced the word) |
| `autocorrect_undone` | Auto-corrections taken back by the backspace or the undo key straight after |
| `picked_typing_left`, `_middle`, `_right`, `_further` | Suggestions tapped while a word was being typed, by place in the row |
| `picked_after_space_left`, `_middle`, `_right`, `_further` | Suggestions tapped right after a space, by place |
| `predicted` | Suggestions tapped with no word being typed: next-word predictions, after a space or not |
| `swiped` | Words swiped |
| `swipe_replaced` | Swiped words replaced straight after: a suggestion tapped for it, a backspace or the undo key |
| `typos_left` | Finished words the user deleted back into within 6 keystrokes: a typo left and then fixed |

Read it from a debug build (run-as needs one):

```bash
adb shell run-as com.jtn0123.audiowhisper.keyboard cat files/typing-counters.txt
```

*Forget learned words* (Settings, Typing) resets every count, deletes the file, and erases the
learned words, the words kept as typed and the tap aim with them. Deleting the file with run-as while the keyboard runs
does not reset it: the next save writes the counts back.

### The dictionary build

The app's `main_en-US.dict` is AOSP's English dictionary rebuilt from that corpus by
`lexicon dictionary` (the script's `dictionary` step): AOSP's words, flags and shortcuts, with
each word's frequency and up to 32 next words a word counted from the training sentences.
`crates/lexicon` holds the pieces: `v202` reads and writes the format (writing AOSP's file back
gives its bytes exactly), `count` counts words and pairs the way the keyboard looks them up, and
`dictionary` turns counts into values on AOSP's scale. The build is reproducible: the same corpus
gives the same bytes, and the script says whether they are the app's. Why this corpus and these
settings, and what would change them: [decisions.md](decisions.md#word-statistics-from-a-public-corpus);
the numbers: [language-model-results.md](language-model-results.md).

### The n-gram model

The app's `main_en-US.lm` is a word trigram model over the dictionary's words, built by
`lexicon model` (the script's `model` step) from the same training sentences and the dictionary
it built: interpolated Kneser-Ney, pairs and triples kept if seen at least three times, each log
probability and backoff quantised to a byte. The build is reproducible like the dictionary's.
`crates/lexicon/src/model` counts (`count`, the same sentence splitting and look-ups the keyboard
uses), smooths and prunes (`smooth`), and writes (`write`).

`crates/ngram` reads it, in the app and on the host: `format` is the file layout (its comment has
the table), `context` turns the words before the cursor into the model's context exactly as the
builder did, and `Model` answers a word's probability after one or two words and the likeliest
next words. It reads straight from the bytes, with every read checked, so a damaged file gives no
answers rather than a panic, and one that doesn't add up doesn't open.

`crates/suggest/src/language_model.rs` is how the session uses it: next-word predictions come
from the model; each typed candidate's language term is redone with the model's lift (how much
likelier the word is after these words than anywhere) on the dictionary's scale; and the swipe
ranker hears the lifted value as its n-gram. The user's learned next words still come first, and
without a model the session is the dictionary alone, as before. The app stores the file
uncompressed and `awnative` reads it from the APK at its offset, so it never passes through the
JVM heap (`LatinLanguageProvider.attachBundledModel`). Why a trigram model and these sizes:
[decisions.md](decisions.md#word-statistics-from-a-public-corpus).

AOSP's original is kept, unchanged, as `crates/latin/tests/fixtures/aosp_main_en-US.dict`. Tests
that pin what the decoder answers (the C++ oracle's spot checks, `LatinImeJniTest`, the
scoreboard's own tests) read that file, so their answers stay pinned; tests that compare two
implementations (the port against the C++, Rust against the Kotlin references) run on both
dictionaries. The app attaches the model when it opens the dictionary; tests attach it only where
they test it (`crates/suggest/tests/language_model.rs`, `LatinSessionModelTest`, the scoreboard's
`--model`), so the decoder's pinned answers and the A1 replay digests, which open the engine
without it, are the dictionary's alone.

## Layout

Everything lives in `lib/rust`, one Gradle module that is also a cargo workspace:

```
lib/rust/
  Cargo.toml, Cargo.lock      the workspace; release builds use LTO and abort on panic
  rust-toolchain.toml         the pinned Rust version, with the Android target
  .cargo/config.toml          16 KB page alignment for Android; cargo's output in build/cargo
  crates/glide/               glide typing's scorer: plain Rust, no unsafe, no JNI
  crates/dictation/           dictation text/session/model-cache/segmentation policy and the mic key
  crates/sherpa-ffi/          owned sherpa-onnx 2.0.1 C API boundary for ASR, VAD and Pocket TTS
  crates/suggest/             typing suggestions, auto-correction, the spell checker, what typing learns
  crates/reader/              reading a page aloud: blocks, selections, scrolls, speech, the speaker key, the screen walk's decisions
  crates/keys/                key/domain models, layout assembly/cache rules, geometry and hit-testing
  crates/gestures/            swipes and the start of a glide
  crates/spacing/             phantom and automatic spaces
  crates/popup/               long-press popups: grid, finger tracking, key order
  crates/shift/               the shift key, automatic capitals, the double-space full stop
  crates/composing/           rule-table composers, including Vietnamese Telex
  crates/emoji/               emoji files, skin tones and `:` suggestions
  crates/extensions/          extension metadata, maintainers, validation and archive import policy
  crates/editor/              UTF-16 editor state, reconciliation, input flags and ordered InputConnection plans
  crates/input/               shared dispatcher, pointer/timer policy, key routes, state/subtype, evaluator and feedback
  crates/nlp/                 provider eligibility, request generations, candidate/inline state, learning and smartbar plans
  crates/settings/            preference defaults/admission, stored codecs/migrations and subtype edit/active policy
  crates/gif/                 the GIF panel: its search field, chips, keys, result cache, recents and GIPHY's JSON and URLs
  crates/latin/               LatinIME's dictionary and typing search, ported from its C++
  crates/chars/               Kotlin's character rules, for crates that judge text as Kotlin did
  crates/awnative/            libawnative.so: the JNI functions Kotlin calls, and nothing else
  crates/boundary/            host-only: generates both sides of the JNI boundary (see below)
  crates/latin-oracle/        test-only: LatinIME's C++ built for this computer, to compare the port with
  crates/scoreboard/          host-only: the typing scoreboard (see above)
  crates/lexicon/             host-only: corpus split, the v202 dictionary writer, the dictionary and n-gram model builds
  crates/ngram/               the n-gram model file and its reader: a word's probability after the words before it
  tools/                      generates crates/chars' tables (generate-char-tables.sh)
  src/main/kotlin/            the Kotlin side: loads the library and wraps each handle
  build/generated/nativeBoundary/kotlin/   the generated Kotlin objects (never checked in)
  src/test/kotlin/            JVM tests that go through JNI
  build.gradle.kts            runs cargo
```

There is one native library, `libawnative.so`, however many crates there are. Each area of logic is
its own crate with no unsafe code (`#![forbid(unsafe_code)]`) and no knowledge of Android or the
JVM, so it is tested with plain `cargo test`. `awnative` owns JNI conversion and turns bad input into a Java exception instead of a crash.
The other shipped unsafe boundary is `sherpa-ffi`, which owns the pinned sherpa C API objects;
its safe API exposes no C pointer. The test-only `latin-oracle` calls C++.

Native objects (an index, a query, a word store) cross JNI as a `Long` handle. The Kotlin wrapper
frees each one with a `java.lang.ref.Cleaner` once it is unreachable; a query is also
`AutoCloseable`, and using one after `close()` throws rather than touching freed memory. An object
that changes and is used from more than one thread (a tap trail, the learned and kept words) sits
behind a `Mutex` (`handle::with_locked`).

Files stay in Kotlin. A store's wrapper reads its file and hands Rust the text, and writes the text
Rust gives back, so Rust never touches the disk and the file's format is in one place.

A long list of strings, such as a dictionary's words, crosses as one string and their lengths
(`jni::cut`). A `String[]` costs several JNI calls a string: 160,000 words that way took longer
than the Kotlin being replaced took to do the work.

## Building

`./gradlew assembleDebug` builds the Rust too; nothing else is needed but
[rustup](https://rustup.rs). The first build installs the pinned toolchain and the Android target
from `rust-toolchain.toml`. If your rustup doesn't install them automatically, run
`rustup toolchain install` in `lib/rust`.

Gradle runs cargo three times:

- `:lib:rust:cargoBuildAndroid` builds `libawnative.so` for arm64 Android, linked with the NDK's
  clang, and packages it as the module's native library.
- `:lib:rust:cargoBuildHost` builds it for this computer, which the JVM unit tests (in `lib/rust`
  and `app`) load. It is an input of those tests, so changing the Rust runs them again.
- `:lib:rust:generateNativeBoundary` writes the generated Kotlin objects (see below) before Kotlin
  compiles.

Gradle finds cargo in `$CARGO_HOME/bin` or `~/.cargo/bin`, because Android Studio doesn't see
the shell's `PATH`.

## The generated boundary

Part of the JNI boundary is generated instead of written by hand. So far that is `ShiftNative`;
the other bindings are still hand-written ([all-rust.md](all-rust.md), phase 2).

`crates/boundary` holds typed declarations, one file for each Kotlin object in
`src/declarations`: each method's name and doc, its parameters' names and types, and its return
type. From that one table it writes both sides, so their names and types can't drift apart:

- **The Kotlin object**: `internal object ShiftNative`, its `RustLibrary.load()` init and a
  `@JvmStatic external fun` for each method. `:lib:rust:generateNativeBoundary` runs
  `cargo run -p boundary` before Kotlin compiles, into `build/generated/nativeBoundary/kotlin`,
  which every variant compiles as a source.
- **The Rust entry points**: an `#[unsafe(no_mangle)] extern "system" fn Java_…` for each method,
  which reads the JNI values, calls a hand-written function in awnative and returns its result,
  or throws its error in Java. awnative's `build.rs` writes them into cargo's `OUT_DIR`, and
  `src/boundary.rs` includes them.

The entry points reach awnative through its build script because they hold `unsafe` JNI code,
which may compile only in awnative; the generator writes that code as text and forbids `unsafe`
itself. `boundary` is a build dependency of awnative, so cargo runs the build script again
whenever a declaration changes, and the library can't be built from old declarations. Gradle
writes the Kotlin again whenever `crates/boundary` changes. Nothing generated is checked in. Each
cargo build keeps a copy of the entry points it compiled in `build/rust/boundary`, for
`checkNativeDeclarations` to read.

A declared method's hand-written function is in the awnative module its class names, called as
the method is in snake case (`ShiftNative.isDoubleSpacePeriod` calls
`crate::shift::is_double_space_period`), and takes and returns plain Rust types:

| Declared | Kotlin | JNI | The Rust function |
|---|---|---|---|
| `Param::Int` | `Int` | `jint` | takes `i32` |
| `Param::Long` | `Long` | `jlong` | takes `i64` |
| `Param::String` | `String` | `jstring` | takes `&[u16]`: UTF-16, lone surrogates kept; a null throws |
| `Return::Boolean` | `Boolean` | `jboolean` | returns `jni::Result<bool>` |
| `Return::Int` | `Int` | `jint` | returns `jni::Result<i32>` |

An `Err` throws in Java, as the hand-written entry points' errors do. Another type needs a variant
of `Param` or `Return`, its spelling on each side, how the entry point converts it, and, for a
return, an implementation of `boundary::Returned` in awnative.

The generated code is laid out as the repo's own: the Rust as rustfmt would lay it out, which
`cargo test -p boundary` checks against rustfmt, and the Kotlin in 120 columns. Neither carries
`@Suppress` or `#[allow]`, and clippy checks the included entry points as it does the rest of
awnative.

To add a method to a generated object:

1. Declare it in `crates/boundary/src/declarations/<module>.rs`: its name, doc, parameters and
   return type. One method of a name: overloads are not supported.
2. Write its `pub fn` in awnative's `<module>.rs`.
3. Build. If the function is missing or its types differ, awnative doesn't compile.

To generate an object that is written by hand today, add `src/declarations/<module>.rs` with its
`Class`, list it in `CLASSES` (`src/declarations.rs`), turn its entry points into plain functions,
and delete the hand-written Kotlin object.

## Checks

```bash
./gradlew :lib:rust:check   # cargo test, clippy with warnings as errors (here, and for Android
                            # with and without the GPU renderer), rustfmt --check
./gradlew test              # every JVM test, including those that go through JNI
./gradlew :app:checkNativeDeclarations   # the debug APK's JNI exports and types against the Kotlin
```

`:app:checkNativeDeclarations` (`scripts/native-declarations-check.py`, also run by
`:lib:rust:check`) fails when a production Kotlin `external fun` has no export in the debug APK's
`libawnative.so`, or an export there has no production declaration: one only tests declare
belongs off the phone (`#[cfg(not(target_os = "android"))]`, as the host-only probes are, or a
fixture feature), and one no Kotlin declares is dead.

It also checks types, which names alone miss: a Rust entry point that takes a `jint` where Kotlin
passes a `Long` still links, and the argument arrives cut short. Each module's debug classes give
every production native method's JVM descriptor, and each of its Rust entry points, hand-written
in awnative or generated, must take and return the JNI types `javac -h` would give it: `jclass`
after the `JNIEnv` for a static method and `jobject` for an instance one, `jlong` for a `Long`,
`jintArray` for an `IntArray`, `jobjectArray` for any array of objects. `i32` passes for `jint`
and `f32` for `jfloat`, since jni-sys defines them so; `bool` does not pass for `jboolean`. The
classes must declare the same native methods the sources do, so none escapes, and overloads are
refused. The generated entry points it reads are the copy the APK's own cargo build compiled.

`scripts/native-declarations-mutation.py` breaks the boundary six ways and requires the check to
fail each, then pass restored: a declaration with no export, an export with no declaration, a
test-only export shipped, a hand-written entry point that reads a `Long` as a `jint`, a generated
declaration whose type changed on one side only, and a renamed entry point. A build can't
regenerate only one side, so that case makes the two runs of the generator disagree (`cargo run`
has debug assertions; awnative's release build script doesn't). `cargo test -p boundary` checks
that the generator writes the same files every time, laid out as rustfmt would.

`./gradlew :lib:rust:oracleCheck` runs the LatinIME C++ comparison through `latin-oracle`; it
needs a C++ toolchain, and CI runs it as its own task.

Or, from `lib/rust`, plain `cargo test`, `cargo clippy --all-targets -- -D warnings` and
`cargo fmt`. CI also runs `cargo audit` there, against RustSec's advisories.

The Vello Hybrid GPU renderer (wgpu over Vulkan, behind the `aw-hybrid` log switch) is built only
on request, with the `hybrid` feature of `render` and `awnative`: `-PhybridRenderer=true` for an
APK. Every other build ignores the switch and draws on the CPU (see docs/decisions.md).

## Rules

- **Same results as the Kotlin.** A port keeps the Kotlin's arithmetic: `Float` stays `f32`,
  `Double` stays `f64`, operations happen in the same order, and Java's `Math.max`/`min`
  (NaN wins; -0 is less than +0) are written out by hand, since Rust's `max`/`min` differ, as is
  Java's `Float.compare` for sorting (-0 before +0, NaN last). On the phone both use bionic's
  `exp`, so costs match to the bit. On a computer the JVM's `Math.exp` can differ from the
  system's in the last bit, so the JVM tests let a swipe word's frequency be one unit in the last
  place off as long as the `Float` the classifier takes of it is the same; the emulator check
  allows none.
- **Judge characters as Kotlin does.** Text crosses JNI as UTF-16 (`&[u16]`), and anything that
  asks what a character is (`isLetter`, `isWhitespace`, `uppercaseChar`, a regex's `\s`) asks
  `crates/chars`, never Rust's `char` methods, which answer differently for some characters.
  Kotlin's answers come from `java.lang.Character`, which differs between a computer's JVM and
  Android (ICU): on Android `isLowerCase` leaves out ª and ʰ, and `\s` takes in the no-break
  spaces. So `chars` has a table for each, generated by asking Java on each; the library built for
  Android uses Android's, and the one the JVM tests load uses the JVM's. They are Unicode 16 (JDK
  24, Android 16). After moving to a JDK or an Android with a newer Unicode, run
  `ANDROID_SERIAL=<emulator> tools/generate-char-tables.sh` in `lib/rust`.
  The same goes for strings: `lowercase()`, `uppercase()`, `equals`/`startsWith(ignoreCase)`,
  `toIntOrNull()` and code points are `chars::case`, `chars::text` and `chars::code_point`, which
  copy each platform's way of doing them (Android compares `ignoreCase` char by char; the JVM, code
  point by code point). `JavaTextTest` holds them up against Java on every character, every code
  point and random strings of the hardest ones; on the emulator, none of 17,792,939 comparisons
  differed. One difference is on purpose: Rust decides whether Σ ends a word (ς) by ICU's rule,
  as Android does, which the JVM's `toLowerCase` doesn't, so the JVM tests count ς and σ as one.
- **Prove it with a differential test.** Before deleting Kotlin, copy it into the tests as a
  reference (as `ReferenceGlideClassifier`) and compare the two over real data and odd cases.
  Make sure the test fails when the Rust is changed slightly, so it isn't passing for nothing.
  The JVM tests see the JVM's character rules, so where those matter, also compare the two on the
  emulator: a temporary patch runs the test's cases there and logs only how many differ.
  Where the code asks something outside it, as the suggester asks the C++ decoder, give both the
  same answers: the test's made-up dictionary answers each question by hashing it, so it doesn't
  matter how often or in what order each side asks. The real decoder isn't so steady: asked for the
  same strip twice, it gave a different one 3 times in 4,000 on the emulator (odd words such as
  "vis-a-vis"), so a comparison over the bundled dictionary remembers each of its answers and hands
  both sides the same ones.
- **Write down every difference that's on purpose**, in the Rust and in the test. The glide
  scorer has two: equally near keys keep layout order (Kotlin used hash order, which changed
  from run to run), and an empty swipe gives nothing (Kotlin scored stale storage). A tap trail
  can't be made with a negative capacity (Kotlin's took one, then threw on the first character
  typed into it). Text can't be cut for speech into pieces of no length or less (Kotlin's cut
  forever for 0 and threw `StringIndexOutOfBoundsException` for less); the app only asks for 3,900.
- **One copy of each number.** A Kotlin wrapper reads the rule's numbers (the QWERTY layout's
  sizes, the correction threshold) from Rust rather than repeating them, so the two can't drift.
- **No file over 500 lines**, generated tables aside. A crate's tests go in `<module>/tests.rs`
  (`#[cfg(test)] mod tests;`) rather than inline once a file grows past that.
- **No warnings, and none silenced.** Clippy runs with `-D warnings`; fix what it says rather than
  adding `#[allow]`.
- **Unsafe stays at native boundaries**: JNI in `awnative`, and sherpa C ownership in the dedicated
  `sherpa-ffi` crate. Logic crates forbid unsafe. Every unsafe block/implementation has a `// SAFETY:` comment.
- **Never hold a lock across a call back into Kotlin.** Production learning and glide context stay in native Rust.
  Reference-only dictionary callbacks acquire store locks only for each read; Android node
  and PCM callbacks must likewise never run under a reentrant native lock.
- **Kotlin that Rust calls back is kept by name.** R8 shrinks the debug build too, and deletes or
  renames what it doesn't see Kotlin call. Rust finds Android callbacks such as `ReaderTree.facts` and `TtsAudio.audio`
  by name, so `consumer-rules.pro` keeps them. Legacy dictionary/name callback exports
  compile only for host/opt-in fixtures; their reference retention rules stay available.
- **A panic is a bug.** Release builds abort on panic and the panic hook writes it to logcat
  (tag `awnative`) first. Bad input from Kotlin is an exception, not a panic.
- **Never log what the user types.** Timings and counts only, as everywhere else.

## Adding a piece

1. Add a crate under `crates/` with the logic and its tests; no JNI, no unsafe.
2. Add its JNI functions: a new awnative module, named after the crate, with a function for each,
   declared in `crates/boundary` (see "The generated boundary"). Where the generator doesn't take
   a type yet, write the entry points in that module and the `external` declarations in
   `src/main/kotlin` by hand. The Kotlin wrapper goes in `src/main/kotlin`.
3. Copy the Kotlin it replaces into the app's tests as a reference and write the differential
   test. Run it, and make it fail once on purpose.
4. Switch the app over, delete the Kotlin, and check it on the emulator.
5. Add it to the table above.
