# Stage 3B package 7: clipboard and GIF/media

The default-off Rust prototype now keeps its surface for clipboard history, the media
panel and GIF search typing. The normal renderer remains Compose. Room, Android clipboard
access, network requests, bitmap/frame decoding and rich-content commits stay in Kotlin.

## Source specification and implementation

The Compose specifications read for this package are:

- `ime/clipboard/ClipboardInputLayout.kt`, `ClipboardHistory.kt`, `ClipboardManager.kt`,
  and `provider/ClipboardDatabase.kt` (including `ClipboardItem`).
- `ime/media/MediaInputLayout.kt`, `ime/media/emoji/EmojiPaletteView.kt` and the existing
  package 6 media controls.
- `com/jtn0123/audiowhisper/gif/GifPanel.kt` (including `GifSearchBar`), `GifSearch.kt`,
  `GifImages.kt`, `GifSender.kt` and `Giphy.kt`.

Clipboard ordering, stable ties, recent/other groups, nullable text duplicates, old/new pin
preservation, expiry, unpinned limits and exact/wildcard MIME eligibility now run in
`stores::clipboard_policy` (`render::clipboard_policy` until [phase 5](phase5-stores.md) moved
it beside the store). The Compose storage path calls the same Rust policy. The JNI transport
preserves Java UTF-16 and integer overflow behavior. Kotlin retains Room and
system clipboard operations; native commands call the existing paste/pin/unpin/delete and
clear-exact handlers, including Compose's different content-file cleanup paths.

The native clipboard panel draws adaptive or fixed-column staggered history, pinned/recent/
other headings, text/image/video items, classifications, filters, locked/empty/disabled
messages, enable/disable controls, the clear confirmation and pin/delete/paste menus.
Room ids retain stable semantic identities across updates. Long preview text scrolls within
the modal independently of the history. Android supplies localized labels, dates, icon
paths and text measurements. Clipboard thumbnails are sampled/decoded on admission and
uploaded as pixels; drawing never decodes them.

The native GIF panel draws search/trending headers, scrollable category chips, three-column
staggered results, loading/error/empty messages, attribution, sending state and the media
bottom controls. Picking a result invokes the same `GifSender.send` path as Compose, with
its `commitContent` / rich-content handling and fallback. `GifDataSource` is injectable for
tests. Production still uses GIPHY when configured.

Android parses/decodes and composites GIF frames once per admission, including disposal
modes and encoded repeat metadata. Rust owns visible-only playback deadlines and an LRU
pixel cache: 24 MiB total, 4 MiB per item, at most 64 frames. Accounting includes both CPU
and Hybrid copies; pixel alpha is premultiplied once. Invisible animations stop scheduling,
and closing the panel releases decoded buffers and preview bytes. Decode jobs finish their
admission rather than cancelling themselves when the first uploaded frame becomes cached.
Failed previews are remembered for the current cohort instead of retrying every paint.
An admitted GIF can finish uploading offscreen, but has no animation deadline there; a
generated-GIF JNI regression reproduces scrolling away between frame uploads.

The search strip and text keys share one Rust surface. The existing GIF key dispatcher
routes typed keys into the Rust-backed query while `GifSearch.typing` is active, keeping
the editor unchanged. Back/search controls, prompt, query, ellipsis and cursor are native.
A transport regression test keeps the evaluated smartbar action cohort intact while the
search strip reserves its row. Android still hosts inline autofill and the previously
retained system/dictation views; no clipboard or GIF panel hands over to Compose with the
prototype enabled.

## GIPHY boundary and fixtures

**No GIPHY key is configured. Live GIPHY search, downloading, decoding and sending are
unverified. No key was requested.** Compose hides its GIF tab without a key; Rust does the
same, leaving the ordinary emoji media view. An instrumentation-only controller activates
search typing for the no-key routing fixture; production does not gain a fake GIF tab.

Tests generate a few small static/animated GIFs locally with Java ImageIO. Nothing is
downloaded. The fake source drives real native result layout, pixel upload, selection,
sending exclusion, loading/empty/error states and panel-close idle behavior. Separate frame
checks cover delays, disposal metadata, truncation and image limits. These tests establish
fixture behavior, not successful live GIPHY delivery or an emulator rich-content send.
The emulator clipboard fixtures contain harmless text only; platform image/video thumbnail
extraction and Android GIF decoding were not visually exercised on that device. The JVM
checks cover generated GIF framing and pixel uploads; Rust checks cover playback/cache state.

## Differential tests and deliberate mutations

Frozen Kotlin references are test-only under `lib/rust/src/test/.../rust/`:

- `ClipboardMatchTest`: 12,000 cohorts for ordering, groups, duplicates, limits, expiry
  and pin preservation, including nullable/lone-surrogate text and Java integer extremes;
  MIME exact/wildcard/text eligibility combinations.
- `ClipboardPresentationMatchTest`: 5,760 grids / 230,400 lane placements across densities,
  widths, fixed/adaptive columns, group membership and item sizes.
- `ClipboardDialogMatchTest`: 270 naturally measured confirmation-column/button layouts.
- `ClipboardMenuMatchTest`: 38,400 filter, selection, popup, pin/unpin, clear and dismissal
  transitions. `ClipboardTouchMatchTest`: 512 minimum-target/sibling-clipping cases.
- `MediaPresentationMatchTest`: 2,400 grids, 80,000 scheduler/cache steps, explicit LRU victim
  identity, finite/infinite repeat counts and 48 category scroll/clamp cases.
- Existing `GifRulesMatchTest` retains the frozen search/query/key/chip/result-cache/recents
  references. `RendererSearchTransportTest` exercises 144 evaluated smartbar cohorts through
  the actual JNI renderer with a bundled test font.

Native tests additionally cover stable Room identities, modal accessibility/actions, disabled
and locked history, independent preview scrolling, cached aspect retention across Room
updates, empty-filter clear eligibility, GIF selection, pixel validation/alpha,
cache/frame budgets, visible deadlines and close-idle behavior. The local fake-source test
uses generated GIFs through the actual native surface API.

Deliberate mutations were restored after the relevant JVM suite failed: ordering direction;
recent boundary; duplicate predicate; limits; old/new pin preservation; expiry; MIME wildcard;
grid columns and lane ties; filter/scroll resets; popup and selection state; clear preserving
pins; minimum touch size; aspect clamp; frame delay/index; visibility; cache limit; encoded
repeat count; chip scroll direction; sending exclusion; query length/key routing; and the
search smartbar action cohort; clear eligibility for an empty filter; and trailing
confirmation-button placement. The first LRU mutation escaped the randomized trace, so a
victim-identity fixture was added; the same mutation then failed. Evidence lives in ignored
`build/stage3b7/mutation-*.json` and corresponding logs. No ignored/deleted tests, warning
suppression or lint baseline was added.

The emulator fixture controller uses Android Instrumentation directly. An earlier
AndroidJUnit controller attempt failed to resolve `kotlin.LazyKt` in the minified debug/test
APK combination; this package does not claim a passing `connectedAndroidTest` suite.
The required Gradle gate runs the JVM suites, and device fixture results are reported
separately. No test was marked ignored or removed.

## Reproduction

Use the runtime id returned by T3 device tools as `ANDROID_SERIAL`; never use the reserved
emulator. Install through `scripts/install-dev.sh`, including the disposable probe built by
`scripts/render-benchmark/build.sh`. The harness accepts only its harmless clipboard
fixtures, refuses a nonfixture history, and records digests/metadata rather than real text.
Snapshot Room and `jetpref_datastore` privately before enabling history. Restore them with
the IME stopped after removing fixture entries and clearing the test primary clipboard.

```
python3 -I scripts/render-benchmark/clipboard-gif.py workflow build/stage3b7/final-workflow
python3 -I scripts/render-benchmark/clipboard-gif.py visuals build/stage3b7/final-visuals --agent "$AGENT_DEVICE"
./gradlew -PrendererFixture=true :app:assembleDebugAndroidTest
scripts/install-dev.sh --probe app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk
adb -s "$ANDROID_SERIAL" shell am instrument -w com.jtn0123.audiowhisper.keyboard.test/com.jtn0123.audiowhisper.render.FixtureInstrumentation
```

While that test-only controller is active, run `clipboard-gif.py search-visuals` against the
same visual directory. Finish it with the `RENDER_FIXTURE` broadcast and `--es op finish`.
`compare-clipboard-gif.py` checks all captured states. It reuses the earlier keyboard
comparator for spoken punctuation/hints, and separately checks panel labels, effective
ancestor-enabled states, clicks and target centers within 3 px. Header icons are matched
to Compose's unlabeled buttons by geometry; modal occlusion and generated timestamps are
recorded explicitly. The comparator's adversarial tests reject wrong labels, actions,
enabled states and displaced targets.

## Local gate and source accounting

`./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug`
exits **0**, **BUILD SUCCESSFUL in 3m 13s**, 346 tasks (344 executed, 2 up-to-date).
`build/stage3b7/local-gate6.exit` captures Gradle's own status directly, without a pipe.
All **697 JVM tests** pass with zero failures/errors/skips. Rust workspace tests, formatting,
clippy with `-D warnings` and the C++ oracle pass. App lint contains only
AndroidGradlePluginVersion, NewerVersionAvailable and ChromeOsAbiSupport; Rust-module lint
has no findings. The test-only controller APK also assembles successfully. Hosted CI is
blocked by account billing; no hosted success is claimed.

Relative to `65c5e7d8`, non-test Kotlin has **91 removed / 522 added** lines (net +431).
Rust has **3,149 added / 124 removed** lines; 2,847 added lines are outside Rust test paths.
Counts include blank/comment lines as git does. The Compose fallback remains, so Android
resource/storage/decoding adapters increase Kotlin while the new rules and drawing move to
Rust. Every changed/new source file is at or below 500 lines.

## Device fixtures and privacy

All device work uses the dedicated `aw_keyboard` runtime **emulator-5556**, through T3 device
tools and the returned agent-device command. The reserved emulator was not used. Installation
uses only `scripts/install-dev.sh`. Evidence stays under ignored `build/stage3b7/`; screenshots,
dumps, APKs, databases, preferences and generated images are not committed.

The original clipboard history had **zero items**, and there were no explicitly stored
clipboard preference keys. Room files and the original preference file were privately saved
before enabling history. Only `aw-test-1` through `aw-test-4` are admitted by the probe writer.
The harness refuses nonfixture history, and its public results contain hashes, lengths,
pin flags and MIME metadata. It disables its accessibility observer while typing or pasting
to avoid perturbing synchronous InputConnection selection reads.

`final-workflow3/comparison.json` reports **match: true**. Pasting history item 2 produces
9 UTF-16 units, cursor 9..9 and SHA-256
`262d1f67d4bc0947e07fae5401d8c0ed6aa5a2b317a62d8585d484fd3f8d1dab`
on both renderers. Pin, unpin, delete and clear produce the same ordered stored records.
Comparison normalizes opaque Room ids and wall-clock creation timestamps; it preserves
content digests, ordering, types, pins and MIME values. Clear leaves the pinned record,
and fixture cleanup removes it and clears the test primary clipboard.


## Visuals, accessibility and search routing

`final-visuals2/` contains 14 accepted screenshot pairs, Compose left / Rust right:

| State | Floris Day | Floris Night |
| --- | --- | --- |
| Clipboard empty | `day-clipboard-empty-pair.png` | `night-clipboard-empty-pair.png` |
| Three text records + one pinned | `day-clipboard-items-pair.png` | `night-clipboard-items-pair.png` |
| Long-press menu | `day-clipboard-menu-pair.png` | `night-clipboard-menu-pair.png` |
| Clear confirmation | `day-clipboard-clear-dialog-pair.png` | `night-clipboard-clear-dialog-pair.png` |
| No-key media view | `day-gif-no-key-pair.png` | `night-gif-no-key-pair.png` |
| Empty GIF search strip | `day-gif-search-empty-pair.png` | `night-gif-search-empty-pair.png` |
| GIF search typing | `day-gif-search-typing-pair.png` | `night-gif-search-typing-pair.png` |

All **14 accessibility comparisons have zero bug verdicts** (`comparison.json`). The
comparison checks named modal controls and excludes grid targets occluded by the modal
scrim; those same grid targets are checked in the ordinary history view. Native enabled
state under the clear dialog matches Compose, including its still-enabled background
history semantics. Twenty-one harness/comparator tests reject wrong labels/actions/states,
displaced targets, disabled-ancestor mistakes, and accidentally exempted modal controls.
The dumps do not establish a separate spoken TalkBack trial.

Search routing matches under both themes: query length 3 and SHA-256
`77af778b51abd4a3c51c5ddd97204a9c3ae614ebccb75a606c3b6865aed6744e`, while
the editor stays at 0 UTF-16 units with empty SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
The test-only controller confirms typing is active and key availability is false. Its
final result is `fixture_complete=true`; it is finished before the ordinary regression.

Representative pairs were visually reviewed. They are **not pixel-identical**: backspace
icon size/strokes, shadows, category-indicator thickness and clear-message wrapping differ.
Fixture timestamps and cursor blink phase differ between sequential captures. The native
cursor uses discrete 530 ms blinking, and native panels do not reproduce Compose's ripple,
overscroll/stretch and transition animations. Decoded images use rectangular clipping;
rounded GIF image corners are not an accepted pixel-parity claim. The live GIF grid is
unverified without a key. The prototype remains default-off.

An earlier capture's clear-dialog JSON was overwritten by cleanup. Its images/dumps were
moved to `rejected-clear-capture/`; the accepted `clipboard-clear-dialog` files have their
own names and were recaptured. Development runtime failures (search smartbar cohort,
unsupported fixture file API, AndroidJUnit setup) are retained separately from accepted
runs. The fixed smartbar path has a JNI/JVM regression test. The instrumentation shell's
normal AndroidRuntime debug/startup/exit messages are archived in
`controller-runtime-bootstrap.log`; none is a fatal or AndroidRuntime error.


## Regression, logcat and cleanup

`final-regression/` passes the ordinary 300-press + eight `e`-hold fixture on **both**
renderers. The 300-press digest is
`809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3`
(300 UTF-16 units, cursor 300..300), and after the holds it is
`c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555`
(308 units, cursor 308..308). The final literal `FATAL|AndroidRuntime` logcat check has
**zero matches**, and render/Compose exception checks have zero matches.

Cleanup restores the original empty Room history and exact clipboard setting values,
including absent keys, from the private backup. The complete preference file was verified
byte-for-byte immediately after restoration. Final preferences confirm prototype **off**,
**Always day + Floris Day**, and the original clipboard key set. Accessibility is restored
to its original unset services / disabled state. The probe and controller test APKs are
removed, the fixture query-digest file is removed, and the project IME remains selected.
DND is off. Streams 1, 2, 3 and 5 are 0; streams 4 and 10 are 1, their reported minimum.
`cleanup/restoration.json` records the checks. T3 `device_close(shutdown=true)` succeeds for
emulator-5556; no other device is closed.

The accepted APK SHA-256 is
`3e38c140074c1a32e5b534ba525a1fe0d2d0bb8a8967506b17cec71c32fd477e`
(`final-apk.json`). These results establish the requested emulator/fixture checks and local
gate. They do not establish live GIPHY delivery, AndroidJUnit connected-suite acceptance,
phone performance or default-on readiness.
