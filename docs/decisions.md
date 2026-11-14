# Decisions

The choices that shape this fork, why each was made, and what would change it. One
paragraph each; the code comments say how, this says why.

## Android 16 (API 36) and up

`projectMinSdk=36` in `gradle.properties`. The keyboard is built for one phone, a Galaxy S24
Ultra on Android 17, not for the Play Store. Requiring Android 16 let us use `Locale.of`
and delete the window-colour, haptic-setting and version checks that only older Android
needed, which is code nobody here could test anyway. **Would change if** the keyboard has to
run on a second, older phone; then lower it to that phone's API level and bring back only
the checks lint asks for.

## GIPHY for GIFs, not Tenor

Tenor's API shut down on 30 June 2026, which left GIPHY and KLIPY (a Tenor-compatible
replacement). GIPHY's API is documented and stable, has a free beta key (100 calls an hour,
which is why the panel calls it only when you ask for something), and serves media from
servers that don't count against that limit. Its terms require the "Powered by GIPHY" label
and say not to cache or store media, which is why previews live only in memory while the
panel is open (`GifImages`). **Would change if** GIPHY's key, limits or terms stop fitting a
personal keyboard, or KLIPY publishes stable documentation.

## arm64 only

`ndk { abiFilters += "arm64-v8a" }` in `app/build.gradle.kts`. The phone is arm64, and so
are the emulators used for testing on this Mac. Each extra ABI means building the Rust,
LatinIME's C++ and sherpa-onnx again, and about 30 MB more APK. Lint's
`ChromeOsAbiSupport` warning is expected and left visible, not suppressed. There's a second,
quieter reason: the Rust typing code is checked bit for bit against the C++ it replaced, and
the two round alike only where both fuse multiply-adds, which is arm64 (see
`.github/workflows/android.yml`). **Would change if** the keyboard needs to run on an x86-64
device; that would also need the oracle tests to accept a last-bit difference there.

## Logic and all UI drawing in Rust

Decided on 2026-11-14: every decision and all application UI drawing belong in Rust;
Kotlin/Java are thin Android adapters only. Adapters transport lifecycle, InputConnection,
surfaces/Choreographer/insets, accessibility, resources/fonts/colours, file/network IO,
audio, Room/content providers, intents, SAF and permissions. They execute native plans and
return platform facts; policy, validation, navigation, editing, layout and semantics live in
Rust. Android-owned picker/permission/inline-autofill UI stays at the platform boundary.
The percentage is reported, never targeted: stage 4 sizing showed complete settings plus
Compose keyboard deletion reaches about 63–67%, so a 90% quota does not describe the desired
ownership. Preserve the behavior and device gates, and keep both Compose fallbacks until
phone acceptance. Do not add Rust or remove tests for a ratio. Stage 5 inventories remaining
policy and true adapters; it does not authorize ports. **Would change if** measured platform
constraints require an explicit exception supported by runtime evidence.

### Historical 90% decision (superseded 2026-11-14)

Decided on 2026-11-12, replacing the 2026-11-08 logic-only decision: aim for at least 90% of
our non-blank main-source lines in Rust, including the keyboard renderer and settings models.
Keep one library (`lib/rust`, one `.so`), with JNI only in `awnative`. Kotlin keeps the IME
service shell, a draw surface and touch forwarding, the TalkBack bridge, microphone,
text-to-speech, accessibility-service and other Android APIs, and the thin settings Activity/platform adapter. The user decision on 2026-11-14
moves settings drawing, navigation/widgets and editing models to the same Rust renderer;
Compose settings stay default/fallback until phone acceptance (see [design](stage4-settings.md)). This removes duplicated policy and shares text rules with the
Mac app; Kotlin is already memory-safe, so safety alone is not the reason. Try pure Rust
Vello + Parley + Skrifa/Fontations first to avoid another large C++ dependency. Use skia-safe
only if the prototype cannot match the look or speed; Skia's compatibility then justifies
its estimated 15–35 MiB. Keep the phone's fonts and emoji, exact behaviour and reviewed
anti-aliasing tolerances. Delete the Compose keyboard after phone acceptance, including
120 Hz trials on the S24 Ultra. `docs/rust.md` defines the stages and acceptance gate.
**Would change if** measured rendering, accessibility, system-font/emoji compatibility or
JNI costs cannot meet that gate; revise the boundary or renderer from measurements before
switching the default, rather than preserving two complete keyboards indefinitely.

## Latin-script input and no developer tools

Decided on 2026-11-12: remove FlorisBoard's developer tools and non-Latin input because the
user types only Latin-script languages. Keep accented Latin layouts, Vietnamese Telex,
numeric, symbol and phone layouts, emoji, the add-on/extension system and the theme editor.
Removing unused providers, packs, composers, layouts and presets reduces the code to port
and maintain. **Would change if** the user needs a non-Latin language or a specific diagnostic
tool; restore that feature deliberately with its tests rather than carrying every upstream tool.

## Pocket TTS on 4 threads for read-aloud

The built-in voice is Kyutai's Pocket TTS (int8) through the sherpa-onnx already bundled for
dictation, so it adds no new native library, speaking as one reference sample ("Loona"). It
runs on 4 threads because the S24 Ultra measured fastest at 2 to 4 for Parakeet: 6 or 8
spill onto the efficiency cores and ran up to 1.8x slower. The emulator spoke 10.8 s of
audio in 8.3 s on 4 threads. When the voice isn't installed or won't load, the phone's own
text-to-speech reads instead. **Would change if** a measurement on the phone
(`scripts/bench-voice.sh`) shows another thread count is faster, or a smaller voice model
sounds as good.

## The debug and personal builds

The debug build is the one used day to day, so it is shrunk by R8 like release, with names
left unobfuscated and line numbers kept. That keeps it fast while still debuggable, and
`run-as` works, which the model install scripts need. The `personal` build is release
(optimised, not debuggable) but signed with the debug key, so it installs over the debug
build and back without losing the app's data. `run-as` doesn't work on it, which
`scripts/install-dev.sh` accounts for. Both install alongside stock FlorisBoard under
`com.jtn0123.audiowhisper.keyboard`. **Would change if** the keyboard is ever distributed;
then it needs a real release key, and the models need installing without a debuggable build
(grade report item E5).


## Retain the Compose root while removing keyboard drawing surfaces

Package 8 keeps the root and bottom-sheet Compose hosts because they still provide the
live theme, preference/resource bindings, background images, sizing/insets and Android
inline-suggestion interop. A plain View factory alone would still load Compose through
those dependencies; removing them together is a separate host rewrite. The current paths
have timestamp-only cold/warm creation measurements in [the package 8 report](stage3b-package8.md).
**Would change if** those remaining adapters can be hosted cleanly with Android lifecycle/
Flow bindings and a measured plain-View trial is neutral or better. This does not block the
35-file fallback keyboard deletion after S24 acceptance, whose dry-run compile already passes.

## sherpa-onnx 2.0.1 C API for dictation and Pocket TTS

ASR, Silero VAD and Pocket TTS now use the exact C API already inside the vendored 2.0.1 AAR, following
its existing binary-vendoring pattern. Gradle restores `libsherpa-onnx-c-api.so` for arm64-v8a
and onnxruntime only; the unused C++ API is excluded. The AAR SHA-256 is
`a8d09d1d32f3eb8520482c6d3d43fae674681c5b050385967c70a3c2c9bf0ee7`, verified against the
[release's published checksum](https://github.com/k2-fsa/sherpa-onnx/releases/download/v2.0.1/checksum.txt)
and GitHub release asset digest. The C library SHA-256 is
`0eadae93c4d595a26584d7eebde7b397b9a36d01eee3ebc7262e0d34df2fce10`; `verifySherpa` pins both
before packaging. No additional binary download is needed. `sherpa-ffi` is the dedicated unsafe
C boundary, with exact versioned layouts, owned streams/results and stop-before-free; JNI stays
only in `awnative`. Normal APKs no longer depend on the AAR's managed classes or package its
JNI library. A verified extraction task copies only the C API and onnxruntime out of the same
pinned AAR. Packaging preserves both payloads byte-for-byte; onnxruntime SHA-256 is
`33847ad43bffe204699fd4a27f7f3603452a8cdaf2f9a44983a0bc31ffcf2da1`. Explicit `ttsFixture`/`dictationFixture` comparison builds retain the original AAR
as a test oracle. The package-12 implementation/local checks are recorded in [the TTS report](stage2-tts.md);
real-model PCM/stress/timing and final device acceptance remain on a documented device-tool hold.
**Would change if** a later release is deliberately pinned and passes the same byte-identical
transcript, segmentation, flush, lifecycle and timing comparisons in [the dictation proof](stage2-dictation.md).
