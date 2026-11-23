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
`tests.rs`, generated tables (`latin/src/chars/tables.rs`, `chars/src/tables/*`), and entirely
test-only crates (`latin-golden`; `latin-oracle` until phase 6 deleted it). Dependencies' native
source is not ours and does not count, including sherpa-onnx, onnxruntime and renderer
dependencies. Until that deletion, the supplied counter's latin-oracle discrepancy was reported
separately.
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
6. A1: native Rust suggest/decoder session; production LatinIME Java glue removed, C++ oracle retained (done; [proof](stage2-a1.md)). Phase 6 turned the oracle into golden files and deleted the C++ ([proof](phase6-oracle-goldens.md)).
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

#### A top-row key's popup cut off (fixed 2026-11-21)

Long-pressing a key on the top row drew its popup with the upper half missing, and with the
Smartbar off drew no popup at all. A popup's top is 2.5 keys above its key's bottom in
portrait and 3 in landscape, and a second row of choices adds 0.4 of that
(`popup_scene::configure`, `popup::grid::extend`). For a top-row key that starts above the
keyboard. The SurfaceView, and so the buffer Rust draws into, was exactly the keyboard's size,
and both the damage and the raster were clipped to it. Compose drew its popups in the window
above the keyboard, over the app. The fault is older than the generated-boundary merge.

The surface now reaches above the keyboard. Rust works out how many rows the highest popup can
need above the scene (`popup_scene::headroom`: every visible key's popup laid out with six
choices, the fewest that take two rows; none under a full-screen panel, which shows no keys).
The buffer keeps those rows at its top and draws the scene that far down (`Scene::headroom`,
`buffer_height`, `buffer_pixels`). The backend adds the shift to every transform; it is a whole
number of rows, so the keyboard's pixels are unchanged. Only the popup draws above the scene's
top. Everything else is clipped to the scene, so the rows stay transparent over the app.
`RustKeyboardView` lays its SurfaceView out from `-headroom` to its own bottom and asks Rust for
the headroom when the scene changes. The view keeps the keyboard's bounds, so Compose's layout,
the touchable region and the insets apps see do not change, and a tap above the keyboard still
reaches the app. A held finger slides up into the popup because Android sends the rest of a
touch to the window it started in. No Compose file changed.

The other fix was to keep the popup inside the surface, moving it down over the keys. That
covers the key being held and the row below it, which neither Compose nor Gboard does.

`render/tests/popup_headroom.rs` lays out every popup of every key in a three-row keyboard,
with and without a strip, in both orientations. Each must fit in the room kept, and the room
must be no more than the highest popup needs. A top-row popup's choices must show their own
colours above the keys, the room beside it must stay clear, the keyboard must draw the same
pixels lower down, and repainting only the damage must match a full repaint as popups open and
close. `KeyboardRendererTest` checks the headroom across JNI. With the headroom forced to 0, 5
of the 6 cargo tests fail. So do 2 with the popup clipped to the keyboard, 3 without the shift,
1 with a full repaint that leaves out the room, and 3 with the damage not moved down.

On the aw_keyboard_ssd emulator (1080x2400, Android 16), in the render probe:

| Check | Before | After |
|---|---|---|
| Long-press e, p, q, Smartbar on | Upper half of each popup cut off | Whole: e's ē ê é over è 3 ë, p's 0, q's 1 |
| The same, Smartbar off | No popup at all, only the pressed key | Whole, wholly over the app |
| Slide from e up to ê, above the keyboard | Not tried | Typed ê |
| Quick press on w | Key highlight, no preview bubble | The same |
| Tap the app 140 px above the keyboard (Settings search, Smartbar off) | Reached the app | Reached the app |
| Tap the app 20 px above the keyboard | Not tried | Reached the app |
| 300 presses and 8 holds of e (`render-benchmark/run.sh`) | Both digests match | Both digests match, unchanged |

One limit. In landscape the keyboard runs full-screen (the extract view above it) unless the
app or the *landscape input* setting says otherwise. There the input view and
`android.R.id.inputArea` wrap the keyboard (`FlorisImeService.updateSoftInputWindowLayoutParameters`),
the wrapping Compose root clips the surface at the keyboard's top, and a top-row popup is still
cut, as before. Fixing that changes the Compose root's layout, which is frozen. The plain-View
root (all-rust phase 3 item 4) should not clip the surface there: no clipping on the root or on
`inputArea`, so the popup draws over the extract view, with the insets and touchable region
still starting at the keyboard. Landscape without full-screen mode, and one-handed mode, are
covered by the cargo tests but were not tried on a device.

#### The key-press preview bubble (added 2026-11-22)

Pressing a character key draws a bubble above it holding the key's label, as the Compose
keyboard did. The bubble appears on press and goes on release. A slide to another key moves it.
When the long-press popup opens, the bubble is hidden. Space, enter, shift, delete and the other
controls get no bubble, nor do the digits of the number and phone pads. The label is the one the
key shows, so shift and caps lock make it upper case. A key with long-press choices shows
Compose's three dots under its label. The *Popup Visibility* setting
(`keyboard__popup_enabled`, "Show popup when you press a key") is on by default, as in Compose.
Turning it off or on takes effect at the next touch.

| | Compose | Gboard | Rust |
|---|---|---|---|
| Which keys | `isSuitableForBasicPopup`: code above space, not CJK space, not a number or phone pad's digits | Characters. Space, enter and delete show none; the key darkens instead | Compose's rule (`press_bubble::suitable`) |
| Size and place | 1.1 by 2.5 desired keys in portrait, 1 by 3 in landscape, centred on the key, ending at its bottom | A circle about one and a half keys wide over the row above, leaving the key showing with its label greyed | The same box as Compose, moved sideways to stay inside the keyboard |
| Label | The key's label in a key-high box at the bubble's top; dots under it if the key has choices | The label, with dots under it if the key has choices | The same as Compose, styled from Snygg's `key-popup-box` and `key-popup-extended-indicator` rules |
| Release | `hide()` at once, no linger | Gone with the release; nothing was left after a 150 ms tap | Gone at once |
| Slide to another key | Shown on the new key | With glide typing on, its default, a slide becomes a glide trail and the bubble goes | Shown on the new key |
| Glide | Hidden | Hidden | Hidden |
| Long press | The bubble's box stays as the base of the extended popup | The bubble first; the choices then replace it and the keys dim, so one box shows | The bubble goes and the popup draws alone, so no second box shows under it |
| Edge keys | Drawn a little past the keyboard's edge (a twentieth of a key for q) and cut there | Moved inward until the circle meets the screen's edge, on both sides | Kept inside the keyboard on both sides, mirrored rows too |

Gboard (15.1.08) was watched on the same emulator, in the same field, as the device checks
below. Its recordings are in `build/press-bubble/gboard/`, not committed. Rust follows Compose's
shape and Gboard's edges: a bubble cut at the screen's edge was a Compose fault, and Gboard
keeps it whole. Gboard's circle over the row above, leaving the key showing, was not copied:
the bubble keeps Compose's box, which is what Snygg themes style.

The data path:

1. Kotlin's `RendererGestureHost.configure` sends the gesture settings before every touch.
   A twelfth value now carries the setting: 1 for on, 0 for off. `GestureConfig::decode`
   accepts 9 to 12 values and reads it as `bubble`.
2. When the theme is applied, `press_bubble::mark` records which keys may have a bubble.
   It uses the keyboard mode and each key's type, by Compose's rule.
3. `KeyboardTouch::event` ends each touch with `hold_bubble`. It puts a bubble on the held key
   when the setting is on and the key is marked. Otherwise it clears it. Release, cancel and the
   glide intercept (`cancel_active`) clear it too. Pointers roll, so at most one character key is
   held, and so at most one bubble shows.
4. The native session styles a new bubble after the touch (`Session::style_bubble`). It reads
   `key-popup-box` from the theme for that key's mode, shift state and code, as the long-press
   popup does. It also reads `key-popup-extended-indicator` when the key has choices. A held
   bubble is styled again when the keys change (shift, caps lock) and when the choices arrive
   with the semantics. Choices arrive after the keys.
5. `Scene::bubble_view` places the bubble with the popup's own preview box
   (`popup_scene::preview`), moved into the keyboard's width. It returns nothing while a popup
   is open.
6. `Raster::bubble` draws it: the box and its shadow, the label and the dots. The headroom
   already leaves room for a top-row key's popup, which is taller than its bubble.
7. Damage: when the bubble's view changes, `damage_regions` adds its old and new boxes, widened
   by exactly as far as the shadow reaches (`shadow_raster::extent`). Keys that change (a label
   under shift) repaint everything, as before.
8. `Raster::draw_frame` (`render/src/bubble_frame.rs`, `render/src/bubble_cache.rs`) draws the
   frame. A first cut repainted the keys under the bubble when it opened and again when it
   went, some 50,000 pixels each time. A second kept and put back the pixels under it and
   clipped the keys out under its opaque middle, but still drew the bubble, its shadow and the
   keys around it on every press: 4 ms on the emulator, twice the CPU gate (below). Now:
   * each bubble is drawn once, alone, over transparent pixels, into a premultiplied image of
     the buffer's pixels its damage covers. An image is found again by all its pixels depend
     on: the label (so shift and caps lock get their own), the key's height, the bubble's
     bounds, its box and dots styles, and the buffer pixels it lands on. The 32 used last are
     kept, about a letter layout's keys, at 4 bytes a pixel of damage. All are dropped when the
     keys move or the keyboard's background changes (another layout or theme); shift keeps
     them. A new size makes a new surface, and with it a new raster with none;
   * a press paints its key's highlight, as before the feature, keeps the pixels under the
     bubble's damage, and lays the image over them with a plain source-over: each row's opaque
     runs are copied and its see-through runs blended. Nothing under the bubble is painted
     again and its shadow is not drawn again;
   * a release puts the kept pixels back and paints the released key. A slide puts them back,
     then keeps and lays at the new key;
   * a frame that paints under a held bubble (another finger's key, the strip) keeps the fresh
     pixels and lays the image over them again;
   * a region inside another one being painted is painted once.

   Every pixel still ends as a whole repaint (`Raster::draw`) draws it, which lays the same
   image the same way. The image is laid after the panel items and the glide trail, which drew
   over the bubble before; see the differences below. The device profile counts the pixels laid
   (`bubble_px`), the images drawn (`bubble_draws`) and the pixels put back (`restored_px`). The
   GPU path (`hybrid`, off by default) still draws the bubble with Vello in the headroom pass.

Kotlin's part is one line in `RendererGestureHost`: it reads the setting and sends it as the
twelfth value. Jetpref preferences are read in Kotlin, and no Kotlin file was added. Everything
else, from which key gets a bubble to its place, style, drawing and damage, is Rust. TalkBack
shows no bubble: touch exploration is handled in Kotlin and never reaches `Touch`, and a
double-tap activates the key through `Semantics::activate`, which sends the key without
touching it.

Proof:

* `render/tests/press_bubble.rs` (11 tests) covers:
  * which keys are suitable;
  * the preview box in both orientations, with and without a strip;
  * every key's bubble and shadow fitting the buffer, for top-row keys at two widths, column
    counts and gaps;
  * edge keys kept on screen on both sides, one-handed widths, and mirrored rows;
  * styles from the theme, including caps lock and the indicator;
  * the twelfth gesture value;
  * the drawing: label box, dots, the headroom clear beside the bubble, and nothing drawn when
    unstyled, unsuitable or under a popup.
* `render/tests/press_bubble_frames.rs` (5 tests) covers `Raster::draw_frame`:
  * nothing painted outside the bubble's damage, and a deep shadow faded out inside it;
  * a key pressed beside a held bubble showing once the bubble goes;
  * the strip painted again under a held top-row bubble: the image laid over the fresh pixels,
    nothing put back or drawn anew, and the new strip showing once the bubble goes;
  * a bubble a whole draw drew, with no pixels kept, going cleanly;
  * a region listed twice still painted.
* `render/tests/press_bubble_cache.rs` (6 tests) covers the kept images:
  * a shifted label drawing its own image while shift keeps the other;
  * a theme change under a held bubble dropping them and painting everything once;
  * another layout (a key narrower, a key more) or a new size dropping them;
  * each part of the look (label, key height, bounds, style, dots) finding its own image;
  * the last 32 kept, one used again kept longest;
  * the shadow blended over what lies under it, not copied.

  The blend and the runs have unit tests of their own in `bubble_cache.rs`.
* `render/tests/press_bubble_touch.rs` (11 tests) drives `KeyboardTouch` through:
  * press and release, and the controls;
  * a tap's two frames: the press paints its key as with no bubble and lays the image once, the
    release puts the kept pixels back, and the next tap draws no image;
  * the setting off and toggled while typing;
  * a slide across keys, and the long-press handover, including leaving the popup for another
    key;
  * rolling fingers with shift;
  * 240 random fast events;
  * cancel, detach and glide;
  * an accessibility click.

  After every step it checks that repainting only the damage matches a full repaint, that every
  pixel that changed lies inside the damage posted to the screen, and that the headroom is clear
  whenever no bubble or popup is up.
* `awnative` `render_bubble_tests` (3 tests) check that the session styles a touch's bubble,
  restyles it for caps lock and for choices, and places it for the orientation.
* `KeyboardRendererTest` (JVM, 2 tests) draws a held key's bubble across JNI only while the
  twelfth value asks for it, and refuses a thirteenth value.
* `scripts/press-bubble-mutations.py` breaks the feature in 58 ways: which keys, the setting,
  the label and its style, the place and clamp, hiding under a popup, release, cancel and glide,
  the damage and the shadow's reach, keeping and putting back the pixels under the bubble,
  painting under a held bubble, painting a region once, each part of a kept image's look, when
  the images are dropped (a theme, a layout or a size, but not shift), how many are kept and
  which go first, the blend and its runs, and the twelfth value across JNI. Every case fails
  fresh assertions in the tests named for it (`press_bubble`, `press_bubble_frames`,
  `press_bubble_cache`, `press_bubble_touch`, `glide_times`, the `bubble_cache` unit tests,
  `render_bubble_tests` or `KeyboardRendererTest`), and the restored source passes all eight.
  Last full run 2026-11-22: 58 of 58 caught.

On the aw_keyboard emulator (1080x2400, Android 16), in the render probe's plain text field.
Every frame of each recording was looked at in a contact sheet, not only the stills, so a
bubble left behind for one frame would show. The recordings and stills are in
`build/press-bubble/device/`, not committed.

| Check | Seen | Stills |
|---|---|---|
| Hold a middle key, a top-row key, the left-edge key and the right-edge key | A bubble over each with its label. The top-row bubble sits in the headroom, the edge bubbles are moved in to the keyboard's edge | `held-middle-g`, `held-top-row-t`, `held-left-edge-q`, `held-right-edge-p` |
| The same with shift | Upper-case labels | `held-shift-middle-G`, `held-shift-top-row-T`, `held-shift-left-edge-Q`, `held-shift-right-edge-P` |
| Space, enter, delete and shift | No bubble | `controls-space-none`, `controls-enter-none`, `controls-delete-none`, `controls-shift-none`, `controls-shifted-G` |
| Slide along the top row, on the letters and on the symbols | The bubble moves to each key, and none is left after release | `slide-on-e`, `slide-moved-to-r`, `slide-symbols-on-1`, `slide-symbols-moved-to-2`, `slide-after-release` |
| The symbols slide, which rested on its last key past the long-press delay | The popup alone, with no bubble under it | `slide-popup-took-over` |
| Long-press e | The bubble, then the popup alone, then nothing after release | `longpress-bubble-first`, `longpress-popup-alone`, `longpress-after-release` |
| *Popup Visibility* off, then the same four keys | No bubble, and the keys still type | `setting-turned-off`, `off-during-first-tap`, `off-during-top-row`, `off-after-taps` |
| Turned back on | Bubbles again at the next touch | `setting-turned-back-on`, `on-again-middle-g`, `on-again-top-row-t`, `on-again-after` |

One tap in the held run reached the keyboard as a press of more than 300 ms, stretched by the
emulator's input injection, and became a long press. The popup replaced the bubble and nothing
was left behind (`held-long-press-took-over-p`). Afterwards the keyboard was the selected input
method again, *Popup Visibility* was back on, the field was cleared and the `aw-profile` log tag
was empty.

Those checks ran on the second cut (below). The held, controls, slide and long-press runs were
recorded again on the kept image, every frame looked at in a contact sheet, and showed the same:
each bubble with its label and shadow, none left after a release or a slide, and the popup
alone after a long press (`build/press-bubble/device-cache/`, not committed). The setting off
and on was not recorded again.

`render-benchmark/run.sh` on the same emulator. Times are milliseconds; the first three columns
cover every frame of a run (1,018 to 1,097 frames), the last two are the medians of a tap's press
frames and of its release frames that brought no new words. The last two rows are the CPU gate's
own trials, which run `run.sh`, main and the kept image alternately
([cpu-p95-results](cpu-p95-results.md#the-key-press-bubbles-kept-image-2026-11-22)).

| Run | Draw p50 | p95 | p99 | Press frame | Release frame |
|---|---|---|---|---|---|
| Main (two runs) | 0.89–1.00 | 3.03–3.41 | 5.48–7.31 | 0.86–0.94 | 0.93–0.94 |
| First cut: the keys under the bubble repainted on press and on release | 3.42 | 9.58 | 16.00 | 6.09 | 4.61 |
| Second cut: the pixels under it kept and put back, the keys under its middle clipped out, the bubble drawn on every press | 1.13 | 5.68 | 8.34 | 4.06 | 0.17 |
| Main, gate trials (two) | 0.766–0.952 | 2.457–2.966 | 5.06–6.38 | 0.78–0.89 | 0.74–0.80 |
| Kept image, gate trials (two) | 0.908–1.091 | 3.032–3.399 | 6.57–7.23 | 0.94–1.11 | 0.82–0.92 |

Every run in the table typed the expected text: the 300 typed units (`809d588b…`) and the 308
with the popup (`c14d19d2…`). The bubble changes no typed text. Runs whose text differed were
discarded: two of the earlier `run.sh` runs (one after run where the emulator stretched two
35 ms taps past the 300 ms long-press delay, and one before run with no tap over 287 ms), and six
of the ten gate trials, on main and the kept image alike, cause unknown (E29).

The kept image costs a press what main's press costs, plus about 0.12 ms. Vello repaints only the
pressed key's box, as on main (encoding 0.26 ms and raster 0.52–0.54 ms, against main's
0.24–0.27 and 0.49–0.54); the 0.12 ms is copying the 46,306 pixels under the bubble's damage and
laying 39,910 pixels of image over them. A release costs main's release plus about 0.06 ms, to
put the 46,306 pixels back. The second cut's release was cheaper, 0.17 ms, because the pixels it
put back already showed the key released; the kept image paints the press's highlight first, as
main does, and so repaints the released key. An image is drawn once per key: 11 in a trial of
300 taps. The gate's frames that the bubble never touches moved more between trials (0.71 to
1.03 ms at the median) than the bubble costs.

**The CPU gate was not met.** Its wall-clock p95 must be at most 2 ms; the kept image measured
3.032 and 3.399 ms. Main measured 2.457 and 2.966 ms on the same emulator that day, where it had
measured 1.107 and 1.257 ms on 2026-11-17: the emulator ran about four times slower, with two
other emulators and other agents' builds on the host, while the host bench's press and release
frames are unchanged. CPU time p95 stayed under 2 ms in all four trials (main 1.26–1.39 ms, the
kept image 1.48–1.49 ms). The same trials on a quiet host are owed (E28), and on the phone (P18).

Where it differs from before the kept image: the image is laid last, after everything Vello
draws, so it covers the panel items and a glide trail that is still fading, which drew over the
bubble before. Compose drew its fading trail over its popup. A glide itself hides the bubble, so
this shows only when a key is pressed while a trail fades.

What remains:

* Full-screen landscape (the extract view above the keyboard) clips a top-row key's bubble at
  the keyboard's top, as it clips the long-press popup. The cause is the same frozen Compose
  root, and so is the fix: the plain-View root
  ([all-rust phase 3](all-rust.md#3-rust-owns-the-keyboard-l), item 4).
* Landscape without full-screen mode and one-handed mode are covered by the cargo tests only
  (E26). TalkBack touch exploration was not run on a device (E27). The phone has not shown the
  bubble, nor timed it (P18).
* The CPU gate with the bubble on, on a quiet host (E28).

#### A label too big for its key (fixed 2026-11-22)

A key label wider than its key was drawn centred on the key at its full size, so it started past
the key's left edge, over its neighbour, and was cut at the key's right edge. The theme editor's
device check found it ([stage4-theme-editor, part 2](stage4-theme-editor.md#device-check-1)): with no font
size of its own, the space key's "English (United States)" was drawn at the plain key's 22sp
and covered the emoji key. The theme set `text-overflow: ellipsis` there, and Rust ignored it. So
did every other label: a hint, a long-press choice and the press bubble's label could all run
out of their boxes the same way.

| | Compose (`SnyggText`) | Gboard (15.1.08) | Rust before | Rust now |
|---|---|---|---|---|
| The space key, no font size of its own | "English (Unite…" inside the key, from its left | One language: a blank space bar. Two: short codes, "EN • CY", small and centred | Past the key's left edge, over the emoji key, cut at its right | "English (Unite…" inside the key, from its left |
| `text-max-lines`, `text-overflow` | Honoured: lines broken at the box's width, cut to the maximum, the last one ended in an ellipsis or clipped | No themes | Ignored | Honoured as Compose did |
| A theme that sets neither | Every line kept, clipped to the box | — | Ignored | One line, squeezed, then shrunk, then ellipsised, as a suggestion fits its slot |
| The press bubble's label, 60sp | By its defaults, clipped to its box; not tried | — | Past the bubble on both sides, onto the keys beside it | Shrunk to fit inside the bubble |

Gboard was watched on the same emulator (`gboard-01-two-languages.png`,
`gboard-02-after-switch.png`). It never draws a full language name on the space bar, so it never
has to fit one. Rust follows Compose, which is what Snygg themes style.

What Rust does now (`render/src/label_fit.rs`):

* A label that fits draws exactly as before. The check compares the layout the raster already
  looks up with the label's box: inside it, and no more lines than `text-max-lines`. The host
  bench fits no label, and on the emulator the Floris themes draw pixel for pixel as on main.
* A label that does not fit is fitted to the box Compose gave its Text: the key less its
  padding for a key's label, the key for its hint, the cell for a long-press choice, and the
  label box for the bubble. It is clipped to that box.
* Where the theme sets `text-max-lines` or `text-overflow`, it is fitted as `SnyggText` did. The
  label breaks into lines at the box's width. It keeps at most `text-max-lines` of them, and with
  an ellipsis at most as many as fit the height. Where text is cut, the last line ends in an
  ellipsis over the rest of its paragraph. The paragraph is as wide as the label or the box,
  whichever is narrower, and `text-align` places each line in it, `start` by the text's
  direction, so right-to-left text starts at the right. A paragraph taller than its box hangs
  from the box's top, as Compose's Text does. `text-overflow: visible` clips to the key, not to
  the label's box.
* Where the theme sets neither, the label stays on one line and is fitted as a suggestion is:
  squeezed to 70% of its width, then shrunk to 75% of its size and to the box's height, then
  ended in an ellipsis. It stays centred. Compose kept every line and clipped them. The Floris
  themes set `text-max-lines: 1` on every key, so this applies to their bubble and their
  long-press choices, which set neither.
* A phone pad's digit keeps its place left of centre, and a hint keeps its corner, while they
  fit.
* The ellipsis is the suggestion strip's (`smartbar_fit::ellipsised`, moved there from
  `smartbar_raster`). The squeeze and shrink are its `fit_text` and `text_scale`. Nothing is
  copied a third time.

The cost: a fit is made once for each label, font size, box, weight and set of rules, and kept;
the raster keeps up to 256, as it keeps layouts (`Raster::label_fits` counts the fits made). A
label that fits costs a comparison. On the Mac, `keyboard_frame_timing` run ten times each,
alternating main and the branch binaries, gave a press p95 median of 0.084 ms on main and 0.085
on the branch, a release 0.178 and 0.177, and a popup 0.179 and 0.177. That is within noise, and
no frame in the bench takes the slow path. The emulator trials are in
[cpu-p95-results](cpu-p95-results.md#a-label-too-big-for-its-key-2026-11-22).

Proof:

* `render/tests/key_label_fit.rs` (15 tests):
  * under every pairing of `text-max-lines` and `text-overflow`, a label too wide draws inside
    its key and nowhere else;
  * a label that fits draws as before and is never fitted;
  * the default: squeezed, shrunk to the height, ellipsised;
  * Compose's rules: lines broken and cut, the ellipsis, the paragraph's width, a too-tall
    paragraph hung from the top, `text-align`;
  * the real Floris Day stylesheet, read for each key's rules, and its space key with no font
    size, which ends in an ellipsis from the key's start;
  * a hint too big kept to its key's top, the key's padding on each side, a phone pad's digit
    and a long-press choice;
  * the fit kept, and made again for a new label, size, theme or key, and kept for a whole
    keyboard of labels too big.
* `render/tests/key_label_fit_scripts.rs` (3 tests) draws Hebrew and CJK with Noto subsets
  taken from the emulator's fonts (`tests/fixtures/NOTO-TEXT-SUBSETS.md`): every rule keeps them
  inside the key, right-to-left lines start at the right, CJK breaks between characters, and the
  bubble keeps a 60sp letter, a long label, Hebrew and CJK inside it.
* `scripts/key-label-fit-mutations.py` breaks the fix in 31 ways: each side of the fit check
  and its line count, the clip and its box, the hung paragraph, keeping the fit, how many are
  kept and what each is kept by, each of Compose's rules, the default's shrink and ellipsis, and
  each place a label is fitted (padding, the phone pad, the hint, the choice, the bubble). Every
  case fails fresh assertions in `key_label_fit` or `key_label_fit_scripts`, and the restored
  source passes both. Last full run 2026-11-22: 31 of 31 caught.

On the aw_keyboard emulator (1080x2400, Android 16), in the render probe. The stills are in
`build/key-label-fit/device/`, not committed.

| Check | Main | Branch |
|---|---|---|
| A theme whose space key has no font size | "English (United Stat", from past the key's left edge over the emoji key, cut at its right (`main-01-space-no-font-size`) | "English (Unite…" inside the key (`branch-01-space-no-font-size`). Its ink spans the same rows as Compose's (`compose-01-space-no-font-size`), from the same left; it ends 5 px further right |
| The same theme with `key-popup-box` at 60sp, holding m, w and y | m's bubble letter ran 249 px of ink onto the key to its left and 196 onto the key to its right; y's ran 122 onto its left | One dark pixel beside m's bubble, none beside y's (`branch-06-bubble-m`, `-w`, `-y`, `-sheet`). Main's 60sp w filled its bubble edge to edge; the branch's is smaller, inside it |
| Floris Day, letters; the symbols; back to letters; Floris Night | `main-02`, `main-03`, `main-03b`, `main-04` | Pixel-identical to main, the whole keyboard (`branch-02` to `branch-04`) |

The bubble was recorded and every frame looked at in a contact sheet. These checks and the CPU
trials ran on the build before the kept fits' bound was raised from 64 to 256, which none of
these screens comes near. Afterwards the test extension was deleted through *View details*, and
the Theme screen shows *Always day*, Floris Day and Floris Night, as before
(`restored-00-theme-screen-after`). The probe's field was cleared and its keyboard draws as before
the checks (`restored-01-probe-after`); AudioWhisper Keyboard is the input method again, the
accessibility services are as they were and the media volume stayed at 0.

What remains:

* A long-press choice too wide for its cell, and Hebrew and CJK labels too wide for their key,
  were not tried on the emulator. The cargo tests draw them (E31).
* The phone has not shown a fitted label (P19).

### A theme's own fonts (2026-11-23)

A theme's `@font` rules now reach the renderer. Kotlin lists each rule's `src` URI and the
absolute path the theme's asset resolver gives it (`RendererThemeFonts.kt`, `themeFontFiles`),
and passes the list with the theme (`KeyboardRenderer.theme`'s `fonts`, a generated boundary
argument). `render::theme_fonts` reads each file once, checks it is a whole font (every table
inside the file; `head`, `hhea`, `hmtx`, `maxp` and `cmap` readable), and keeps each distinct
family in a process-wide registry under a `Font::Theme(id)`. An element whose `font-family`
names the rule draws its key labels, Smartbar text and panels in that family first, then in
exactly today's chain for any glyph it lacks. Each face is registered with the weight and style
its `@font` set declares, so `font-weight` and `font-style` pick the face as Compose's
`FontFamily` does.

* A theme with no `@font` file draws byte-identical to before: the default fonts' code paths
  are unchanged and no golden was re-recorded.
* A missing, unreadable, too big (over 32 MiB) or broken file leaves its rule without that face;
  a rule with no face left is no font, and its elements draw in the default chain. Nothing
  panics.
* Not done: no synthetic bold or italic for a theme font that lacks the face, where Compose
  would synthesise one; the nearest declared face draws instead. The registry keeps up to 64
  families for the life of the process.

Proof:

* `render/tests/theme_fonts/` (13 tests) draws with "Fixture Script", a Latin subset of Dancing
  Script (OFL) taken from the Android 9 SDK's fonts and renamed
  (`tests/fixtures/THEME-FONT-SUBSETS.md`):
  * the key changes 1,957 pixels and the Smartbar 1,594; the chosen-face report gives "Hello "
    in Fixture Script Regular and "7?", which it lacks, in Roboto Light; bold picks the bold
    face; the default chain draws byte-identical after a theme font is registered; panels inherit
    the font;
  * nine broken files (truncated, header only, 16 bytes short, garbage, garbage behind a header,
    empty, no `cmap`, missing, a directory) each read as no font and draw pixel-identical to no
    font at all; an unresolved URI and an oversized file are not read; one broken face leaves
    the rest of the family.
* `awnative` `render_theme` tests draw the landscape extract view's action label through the
  session with and without the files, and refuse a URI without a path.
* `RendererThemeFontsTest` (JVM, 2 tests) resolves the files as Kotlin does and draws them
  through the generated `KeyboardRenderer`.
* `scripts/theme-fonts-mutations.py` breaks the feature in 27 ways: the theme ignoring its
  fonts, the chain's order, registration, declared weight and style, reading the `@font` sets,
  one id per distinct family, every whole-font check, the session's plumbing and Kotlin's list.
  Every case fails fresh assertions and the restored source passes. Last full run 2026-11-23:
  27 of 27 caught.

The cost, on the Mac: `keyboard_frame_timing` (a theme without fonts) run ten times each,
alternating main's and the branch's binaries, gave a press p95 median of 0.104 ms on main and
0.097 on the branch, a release 0.224 and 0.202, and a popup 0.203 and 0.217. The minimums are
equal; that is noise. Drawing in a theme font was not benchmarked, and neither the emulator nor
the phone has drawn one yet.

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
| Clipboard history ordering, duplicates, expiry, limits, pins and MIME eligibility (stage 2 package 8) | `crates/stores` (`clipboard_policy`; in `crates/render` until [phase 5](phase5-stores.md)) | `ClipboardManager` / `ClipboardHistory` policy; Android clipboard remains Kotlin, and since phase 5 the history itself is the Rust store, not Room | `ClipboardMatchTest` against test-only `ReferenceClipboard`, 12,000 cohorts and MIME combinations, deliberate mutations; paired paste/pin/unpin/delete/clear stored-state fixture. See [package 7](stage3b-package7.md) |
| Snygg compilation, cascade and editor model | `crates/snygg` (`resolve`, `editor`, `spec`, `json`, `json_write`) | `SnyggTheme` compilation/querying, property application/inheritance, built-in declarations, the theme editor's rule ordering/rename policy, and writing a stylesheet as Compose's `PrettyPrintConfig` did (`SettingsThemeEditorMatchTest`, [stage 4 theme editor](stage4-theme-editor.md)) | JVM reference matches: 868,680 bundled-theme queries, 30,000 seeded queries and 7,200 editor steps; native JSON-to-style light/dark prototype oracle. Reversing cascade traversal fails. Five visual pairs are identical and editor edits persist; see stage 2.5 |
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

The JVM differential tests retained the replaced Kotlin as `ReferenceKeySelectors`,
`ReferenceKeyboardState`, `ReferenceKeyMetadata`, `ReferenceKeyCodes` and `ReferenceKeyRules`
(phase 6 swapped them for golden digests of their answers ([record](phase6-kotlin-references.md))):

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
`ReferenceLayoutAssembly` and the old cache policy in `ReferenceTextKeyboardCache` (phase 6
swapped the cache copy for a golden digest ([record](phase6-kotlin-references.md))). It checks:

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

The C++ was ported against itself rather than a Kotlin copy. Until phase 6, `crates/latin-oracle`
built LatinIME's own sources (`lib/latinime`) for the computer running the tests (with the NDK's
`jni.h` and a fake JNI environment that ran the production `JNI_OnLoad`), so a test could put the
same question to the C++ and to the Rust: open the bundled dictionary, a word's probability, an
n-gram's, its word list, and a typing decode with taps and previous words. `cargo run --release
-p latin-oracle --example oracle -- gen 20000 20261109` wrote seeded questions shaped like the
keyboard's, and `oracle run` answered them.

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
the bit. Each module was tested against the C++ through the oracle, and each was broken on purpose,
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

Since phase 6 (2026-11-21) the C++'s answers are golden files: each test records its answers on
a tape (`crates/latin-golden`) that checks them against `tests/golden/*.txt` in `crates/latin`
and `crates/suggest`, in the everyday `cargo test`, and fails naming the first range of answers
that differs. `lib/latinime` and `latin-oracle` are deleted.
[The phase 6 record](phase6-oracle-goldens.md) names the commit to check out to write the goldens
again, and shows they kill every mutation the oracle killed. LatinIME's copyright notice is kept
in `crates/latin/NOTICE`.

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
that pin what the decoder answers (`LatinImeJniTest`, the scoreboard's own tests) read that file,
so their answers stay pinned; tests that compare two implementations (the port against the C++'s
golden answers, Rust against the Kotlin references) run on both dictionaries. The app attaches
the model when it opens the dictionary; tests attach it only where they test it
(`crates/suggest/tests/language_model.rs`, `LatinSessionModelTest`, the scoreboard's `--model`),
so the decoder's pinned answers and the A1 replay digests, which open the engine without it, are
the dictionary's alone.

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
  crates/latin-golden/        test-only: LatinIME's C++ answers as golden files, and the seeded questions
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
its safe API exposes no C pointer.

Native objects (an index, a query, a word store) cross JNI as a `Long` handle. The Kotlin wrapper
frees each one with a `java.lang.ref.Cleaner` once it is unreachable; a query is also
`AutoCloseable`, and using one after `close()` throws rather than touching freed memory. An object
that changes and is used from more than one thread (a tap trail, the learned and kept words) sits
behind a `Mutex` (`handle::locked`).

Files stay in Kotlin. A store's wrapper reads its file and hands Rust the text, and writes the text
Rust gives back, so Rust never touches the disk and the file's format is in one place.

The keyboard's own stored data is the exception ([phase 5](phase5-stores.md)): `crates/stores`
reads and writes the clipboard history, its media files' records and the user dictionary itself.
Each change replaces the whole file, and a crash at any step must leave the old file or the new
one: write a temporary file, sync it, rename it, sync the directory. Splitting that across JNI
would put half of each save on each side, and the crash tests in `crates/stores` could not reach
the half in Kotlin. So Rust owns the steps and the tests drive them through a disk that can stop
at any one. Kotlin still owns every byte that isn't a record: the clipboard's media copies, the
`ContentResolver` and the system clipboard.

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

Every production native method is generated. So is the Kotlin over most of them. Only test
fixtures declare an `external fun` by hand, and `checkNativeDeclarations` fails on a hand-written
one in a `main` or `debug` source set. Every operation code and packet field the `call`-style entry
points pass is generated too ([below](#operation-codes-and-packets-crateswire)).

`crates/boundary` holds typed declarations in `src/declarations/<module>.rs`, listed in
`src/declarations.rs`. From that one table it writes both sides, so names and types can't drift
apart. It writes four things:

- **The native objects** (`CLASSES`): an `internal object FooNative` with `RustLibrary.load()` and
  an `external fun` for each method. An object whose methods are themselves the API is public
  (`public: true`).
- **The Rust entry points**: an `#[unsafe(no_mangle)] extern "system" fn Java_…` for each method.
  It reads the JNI values, calls a plain function in awnative, and returns its result or throws its
  error in Java.
- **The Kotlin API** (`APIS`): an `object`, or a class that owns a native handle. Each member
  converts Kotlin types and calls one native method.
- **The types the API takes and returns** (`TYPES`, or `nested` in an API): records, enums,
  bit-packed value classes and callback interfaces.

`:lib:rust:generateNativeBoundary` runs `cargo run -p boundary` before Kotlin compiles, into
`build/generated/nativeBoundary/kotlin`, which every variant compiles as a source. awnative's
`build.rs` writes the entry points into cargo's `OUT_DIR`, and `src/boundary.rs` includes them.

The entry points reach awnative through its build script because they hold `unsafe` JNI code,
which may compile only in awnative; the generator writes that code as text and forbids `unsafe`
itself. `boundary` is a build dependency of awnative, so cargo runs the build script again
whenever a declaration changes, and the library can't be built from old declarations. Gradle
writes the Kotlin again whenever `crates/boundary` changes. Nothing generated is checked in. Each
cargo build keeps a copy of the entry points it compiled in `build/rust/boundary`, for
`checkNativeDeclarations` to read.

The generated code is laid out as the repo's own: the Rust as rustfmt would lay it out, which
`cargo test -p boundary` checks against rustfmt, and the Kotlin in 120 columns. The same tests
check that the output is the same on every run. Neither side carries `@Suppress` or `#[allow]`,
and clippy checks the included entry points as it does the rest of awnative.

### Adding a native method

1. Declare it in its class's `methods`: name, doc, parameters and return type. One method of a
   name: overloads are not supported.
2. Write its `pub fn` in the awnative module the class names, called as the method is in snake
   case (`ShiftNative.isDoubleSpacePeriod` calls `crate::shift::is_double_space_period`). Set
   `rust` to call another: a path under the module, `crate::…` for another module, or `::…` for
   another crate's function.
3. If the API wraps it, add a `Member` (below).
4. Build. If the function is missing or its types differ, awnative doesn't compile.

The function takes plain Rust types and returns `jni::Result<T>`, or `T` if it can't fail. One
that returns nothing still returns `jni::Result<()>`. An `Err` throws in Java.

| Declared | Kotlin | The Rust function |
|---|---|---|
| `Param::Boolean`, `Char`, `Int`, `Long`, `Float` | the same | takes `bool`, `u16`, `i32`, `i64`, `f32` |
| `Param::String`, `NullableString` | `String`, `String?` | takes `&[u16]`, `Option<&[u16]>`: UTF-16, lone surrogates kept |
| `Param::Array(Element::Int)`, `NullableArray(…)` | `IntArray`, `IntArray?` | takes `&[i32]`, `Option<&[i32]>` |
| `Param::Strings`, `StringVararg`, `NullableStrings` | `Array<String>`, `Array<out String>`, `Array<String?>` | takes `&[Vec<u16>]`, or `&[Option<Vec<u16>>]` |
| `Param::Out(Element::Int)` | `IntArray` the caller reads | takes `&Env` and the JNI array, to fill with `Env::write` |
| `Param::Object("Kotlin.Type")` | a callback or any object | takes `&Env` and a `jobject`, to call back with `Env::method` and `call_*` |
| `Return::Boolean`, `Int`, `Long`, `Float`, `Double` | the same | returns `bool`, `i32`, `i64`, `f32`, `f64` |
| `Return::String`, `NullableString` | `String`, `String?` | returns `Vec<u16>`, `Option<Vec<u16>>` |
| `Return::Array(Element::Int)`, `NullableArray(…)` | `IntArray`, `IntArray?` | returns `Vec<i32>`, an `Option`; `boundary::Chars` for a `CharArray` |
| `Return::Strings`, `StringArrays`, `IntArrays` | `Array<String>`, `Array<Array<String>>`, `Array<IntArray>` | returns `Vec<Vec<u16>>`, `Vec<Vec<Vec<u16>>>`, `Vec<Vec<i32>>` |
| `OBJECTS`, `NULLABLE_OBJECTS` | `Array<Any>`, read as one record of arrays | takes `&Env`, returns the array `Env::make_objects` made |
| `Return::Object("Kotlin", "jni")` | any other object | takes `&Env`, returns the JNI reference it made |

A null where none is allowed throws `NullPointerException`. A number in an `OBJECTS` record
crosses as a one-element array, so no number is boxed.

Some methods need more:

- **More than seven arguments**, counting `&Env`: the function takes one struct,
  `crate::boundary::args::{Class}{Method}`, with a field for each parameter.
- **A member of the object** (`instance: true`), not `@JvmStatic`: the entry point takes the
  object, a `jobject`, not the class.
- **An `internal` method** (`internal: true`), for a hand-written extension in its module. It
  keeps its JNI symbol with `@JvmName`.
- **Default arguments** (`defaults`): Kotlin values, such as `("vsync", "-1")`.
- **The environment** (`env: true`): the function takes the call's `&Env` first. A
  `Param::Out`, `Param::Object` or object return implies it.
- **A callback** is a `Type::Interface`. Its doc and members are Kotlin; one `fun` member makes it a
  `fun interface`, so Kotlin passes a lambda. R8 can't see the calls from Rust, so it also needs a
  keep rule in `lib/rust/consumer-rules.pro`.

### Adding an API member

An `Api` is `Shape::Object`, or `Shape::Handle` for a class that owns a native handle. A handle
class names its `create` and `free` methods. With `close`, it is `AutoCloseable`: `close()` frees
the handle and later calls get 0, which the native side refuses (`handle::object` throws "the …
is closed"). With `synchronized`, every member and `close()` take the object's lock; without it,
each member keeps the object reachable until its call returns. `params` and `keeps` give the
constructor's parameters; `lend` lets other APIs borrow the handle under the same lock.

A `Member` calls one native method, of its own name unless `call` names another. It is a `fun`, a
`val` read once (`Kind::Val`) or a `val` read on first use (`Kind::Lazy`). `companion` puts it in
the companion object.

Its `args` say how a parameter differs from the native method's, matched by name:

| `ArgKind` | The member takes | The native method gets |
|---|---|---|
| `Same` | the same type | it |
| `Code("WireEnum")` | a `crates/wire` enum | its `code` |
| `Ordinal("Enum")` | a Kotlin enum | its `ordinal` |
| `List` | a `List<String>` | `toTypedArray()` |
| `Text` | a `CharSequence` | `toString()` |
| `Handle("Api")`, `NullableHandle("Api")` | another handle class | its handle, or 0 for null |
| `Lent("Api")` | a lending handle class, or null | its handle under its lock, or -1 |
| `Fixed("expr")` | nothing | that Kotlin expression |
| `As("decl", "expr")` | the Kotlin parameter `decl` | `expr` of it |

`default(name, value)` changes only a parameter's default.

Its `result` says how the native result reads:

| `Out` | The member returns |
|---|---|
| `Same` | the result |
| `List` | an array as a `List` |
| `Entry("Enum")`, `NullableEntry("Enum")` | an `Int` as an enum entry, or null for a negative one |
| `Wrap("Type")`, `NullableWrap("Type")` | the result passed to that constructor, or null for 0 or null |
| `Record("Type")`, `Records("Type")` | a row read as one record, or as a list of them |
| `Pair("A", "B")` | a row of two as a `Pair` |
| `Then("Type", ".kotlin")` | the result with that Kotlin after it, such as `[0]` |

A `Type::Record` lists its fields and `constants`, and is a `data class` with `data`. A
`Type::Bits` is a value class over an `Int` or a `Long`, a property for each run of bits.

### What stays hand-written

283 counted lines in 13 files stay in `lib/rust/src/main/kotlin` (2,338 in 66 before). None
declares an `external fun`. Each holds something the generator has no shape for:

- **`EditorSession` (50)**: a coroutine lock held across suspending transactions, with
  `EditorAttributes`, whose queries each pack their own row.
- **`SettingsRules` (48), `ExtensionPolicy` (30), `BackupPolicy` (21)**: messages over one `call`
  per crate, each reading its own `crates/wire` rows.
- **`SherpaAsr` (39)**: a recognizer is cancelled while it decodes, under a read-write lock, and
  `SherpaVad` reads its phrases from an array of arrays, which no generated record casts without an
  unchecked warning.
- **`NativeRuntime` (24), `RustLibrary` (9)**: loading the library, and the cleaner every handle
  class uses.
- **`Popup` (18), `LayoutRules` (13), `LayoutCache` (13), `SwipeUserWords` (10), `Giphy` (5),
  `Gif` (3)**: Kotlin over generated APIs, in generic or Kotlin-only code.

The test-only natives stay hand-written too: `SemanticRules`, `A1Replay`, `LegacyFixtureNative`,
`DictationSegmentationMatchTest`, `RenderThemeMatchTest` and `TtsComparison`. Their exports are
left out of the phone's library.

## Operation codes and packets: `crates/wire`

Many entry points take an operation and a packet of strings, such as `NlpNative.call(handle, op,
fields)`. Their operation codes, their packets' field positions and the names they pass are
declared once, in `crates/wire`. Rust matches on the declarations. Kotlin is generated from them.
Nothing is copied by hand, and no regex reads Rust source.

One module for each entry point:

| Module | Entry point | Kotlin |
|---|---|---|
| `flag` | every packet's booleans | `WireFlag` |
| `render`, `input` | the renderer's and the input dispatcher's codes | `RenderProtocol`, `InputProtocol` |
| `extensions` | `ExtensionPolicyNative.call` | `ExtensionOp` and its packets |
| `settings` | `SettingsNative.call` | `SettingsOp` and its packets |
| `utilities` | `UtilityNative.call`, in lib/kotlin | `UtilityOp` and its packets |
| `backup` | `BackupNative.call` | `BackupOp` and its packets |
| `editor` | `EditorNative.call` and `.attributes` | `EditorOp`, `EditorAttributeOp` and their packets |
| `nlp` | `NlpNative.call` | `NlpOp` and its packets |
| `dictation` | `DictationSessionNative.call` and `.model` | `DictationEvent`, `DictationModelOp` and the plan |
| `playback` | `PlaybackNative.call` and `.system` | `PlaybackOp`, `PlaybackSpeechOp` and their rows |
| `settings_ui` | `RenderNative.settings` | `SettingsUiOp`, `SettingsUiEdit` and the reply; in `settings_ui::files`, the file picker and file work (`SettingsUiFileEffect`, `SettingsUiFiles`, `SettingsUiPick`, `SettingsUiWork`, `SettingsUiDone` and their codes) |

### The forms

`wire::declare!` declares one group at a time. Each constant is a plain Rust `const`, and the
group is a `wire::Group` the generator reads.

- `enum NlpOp(str) { CONTEXT = "context", … }`: operations Kotlin sends. Kotlin gets an
  `enum class NlpOp(val code: String)`. Each typed wrapper takes the enum and passes `code`, so a
  call can't be given a bare number, a misspelt name or another family's operation.
- `object DictationEffect(str) { CHECK = "check", … }`: codes Kotlin reads or writes as plain
  values. Kotlin gets a Java class of `static final` constants, because Kotlin can import a Java
  class's members with `*` and not an `object`'s. A `when` can use them as branches. Groups of
  one name gather into one class, as `RenderProtocol` does.
- `fields NlpSnapshot { GENERATION, EXPANDED, … }`: a packet's field positions, from 0 in order.
  Rust gets `usize` constants and `SIZE`. Kotlin gets an `object` of `const val`s, `SIZE`, and
  `row(FIELD to value, …)`, which fails if a field is given twice or missing.
- `flag WireFlag { TRUE = "1", FALSE = "0" }`: how a boolean crosses.

A group whose names clash with another's goes in its own submodule, since every constant lands in
the module that declares it.

### Booleans

A boolean crosses as `"1"` or `"0"`. Rust has `wire::flag::encode`, `utf16` and `decode`; Kotlin
has `WireFlag.encode` and `WireFlag.decode`. Only `"1"` decodes as true.

### Why a leaf crate

The domain crates (`nlp`, `editor`, `dictation`, …) need the constants to match on, and they
build for Android. The generator needs them too, on the host. `wire` has no dependencies, so
both can depend on it and neither depends on the other:

- The domain crates don't become dependencies of the generator. `cargo run -p boundary` stays
  small and doesn't rebuild `render` or `nlp` whenever Gradle writes the Kotlin again.
- The generator's code doesn't reach the domain crates or the phone's library.
- awnative's build script doesn't change. The Rust side needs no generated code: the constants
  are ordinary `const`s, compiled where they are used. Only the Kotlin is generated, by the same
  `:lib:rust:generateNativeBoundary` task that writes `ShiftNative`.

Declaring them in each domain crate would make the generator depend on all of them. Declaring
them in `boundary` would make the domain crates depend on the generator.

### What fails, and when

- Two constants of one group with one value: `declare!` runs `wire::check` in a `const`, so the
  build fails ("two constants of one group share a value"). So does a name Kotlin would have to
  escape.
- Two of one name: Rust's own duplicate-definition error. Names gathered into one Kotlin class
  from several groups are checked by `cargo test -p boundary`.
- A renamed constant: whichever side still uses the old name doesn't compile.
- A renumbered constant: both sides follow, since neither spells the value.

`scripts/wire-mutation.py` makes each of these changes, requires the build to fail (or, for the
renumbering, to pass with the new values in the generated Kotlin), and restores the source. Its
last run:

| Case | Change | Command | Mutant exit | Restored exit |
|---|---|---|---|---|
| duplicate-code | playback `BEGINS = 1` → `0` | `cargo check -p wire` | 101 | 0 |
| duplicate-name | playback `ENDS` → `BEGINS` | `cargo check -p wire` | 101 | 0 |
| renamed-kotlin-behind | `PRESS` → `TAP` in wire and dictation | `:app:compileDebugKotlin` | 1 | 0 |
| renamed-rust-behind | `PRESS` → `TAP` in wire only | `cargo check -p dictation` | 101 | 0 |
| renumbered-followed | `HEARD = 30`, `PRESS = "tap"` | reader and dictation JVM tests | 0 | 0 |

The failing cases fail for the stated reason: the log must hold the error quoted above, or
"Unresolved reference 'PRESS'" in `DictationSession.kt`. Run it as
`python3 -I scripts/wire-mutation.py <empty output dir> [case…]`.

### Adding an operation or a packet field

1. Add the constant to its group in `crates/wire/src/<family>.rs`, with a doc comment. A field
   goes where it falls in the packet; the fields after it move, and both sides follow.
2. Handle it in the Rust that answers the call: a `match` arm on the constant, or
   `row[FIELD] = …` in a row made as `vec![vec![]; SIZE]`.
3. Build. Gradle writes the Kotlin again. Call `session.call(NlpOp.NEW, …)`, build a packet with
   `with(NlpThing) { row(FIELD to value) }`, and read one with `result[NlpThing.FIELD]`.

For a new entry point, add `crates/wire/src/<family>.rs` with its `GROUPS`, a `pub mod` in
`wire`'s `lib.rs`, an entry in `WIRE` (`crates/boundary/src/declarations.rs`) naming the Gradle
module that calls it, and `wire` as a dependency of the crate that answers it. lib/kotlin can't
see lib/rust, so groups it calls with are generated there, with its own internal `WireFlag`.

## Checks

```bash
./gradlew :lib:rust:check   # cargo test and clippy (warnings as errors) on every crate (here,
                            # and awnative for Android with and without the GPU renderer),
                            # rustfmt --check, and rustdoc with warnings as errors
./gradlew test              # every JVM test, including those that go through JNI
./gradlew :app:checkNativeDeclarations   # the debug APK's JNI exports and types against the Kotlin
./gradlew :app:checkMutationCases        # every mutation script's cases still apply
```

`:app:checkNativeDeclarations` (`scripts/native-declarations-check.py`, also run by
`:lib:rust:check`) fails when a production Kotlin `external fun` is written by hand rather than
generated, or has no export in the debug APK's `libawnative.so`, or an export there has no
production declaration: one only tests declare belongs off the phone
(`#[cfg(not(target_os = "android"))]`, as the host-only probes are, or a fixture feature), and one
no Kotlin declares is dead.

It also checks types, which names alone miss: a Rust entry point that takes a `jint` where Kotlin
passes a `Long` still links, and the argument arrives cut short. Each module's debug classes give
every production native method's JVM descriptor, and each of its Rust entry points must take and
return the JNI types `javac -h` would give it: `jclass` after the `JNIEnv` for a static method and
`jobject` for an instance one, `jlong` for a `Long`, `jintArray` for an `IntArray`,
`jobjectArray` for any array of objects. `i32` passes for `jint` and `f32` for `jfloat`, since
jni-sys defines them so; `bool` does not pass for `jboolean`. The classes must declare the same
native methods the sources do, so none escapes, and overloads are refused. The generated entry points it reads are the copy the APK's own cargo build compiled.

`scripts/native-declarations-mutation.py` breaks the boundary ten ways and requires the check to
fail each, then pass restored: a hand-written declaration, an export with no declaration, a
test-only export shipped, a generated declaration whose type changed on one side only, a
generated method renamed on one side only, and one for each kind of entry point whose JNI types
javac spells apart from the rest: a callback object, an array of objects, an array of int arrays,
a char array, and an instance method's receiver. A build can't regenerate only one side, so the
one-side cases make the two runs of the generator disagree (`cargo run` has debug assertions;
awnative's release build script doesn't). The last five spell one JNI type differently in the
generated Rust only, which still compiles, since every JNI reference type is an alias of
`jobject`. `cargo test -p boundary` checks that the generator writes the same files every time,
laid out as rustfmt would.

`:app:checkMutationCases` (`scripts/mutation_cases.py`) checks, in well under a second and
without building or changing anything, that every case of every `scripts/*-mutation*.py` still
applies, the cases under `phase6_mutation_cases/` and `settings_ui_mutation_cases/` included: each
file a case edits exists, and each search text occurs as often as the case expects (once, unless it
says otherwise), found in turn as the script applies them. A case whose line has moved would
otherwise stop its script's full run at that case. Each script builds its cases as `Edit`s in
`check_edits()`, from the tables its run reads, and calls `mutation_cases.check_flag()` first, so
`python3 -I scripts/X-mutations.py --check` checks one script. A new mutation script does the same,
or the check fails it, and writes each mutant and restore with `mutation_cases.write()`, which
stamps the file past any build before it: on a filesystem with whole-second times, as the HFS+ of
`/Volumes/2TB`, a plain write in the second a build ran is not rebuilt, and the run tests the
wrong binary. A case known to be stale goes in the script's `STALE`, with why and who is
fixing it; the check fails on a `STALE` case that applies again and on a `STALE` name that is no
case, so the list can neither hide a new failure nor outlive the fix. The check says only that a
case can be applied; whether the tests catch it takes a run of the script.

Or, from `lib/rust`, plain `cargo test --workspace`, `cargo clippy --workspace --all-targets --
-D warnings`, `cargo fmt` and `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`. CI also runs `cargo audit` there, against RustSec's advisories.

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
   declared in `crates/boundary`, with the Kotlin API over them (see "The generated boundary").
   Where the generator has no shape for something, add the shape rather than writing an
   `external fun` by hand: the check refuses one. Kotlin the generator can't express goes in
   `src/main/kotlin`, over the generated API.
3. Copy the Kotlin it replaces into the app's tests as a reference and write the differential
   test. Run it, and make it fail once on purpose.
4. Switch the app over, delete the Kotlin, and check it on the emulator.
5. Add it to the table above.
