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

## Settings prototype parity and bootstrap (2026-11-14, follow-up 2)

Material controls use Compose's Material 3 dimensions and the same resolved ColorScheme
roles; anti-aliasing is the only permitted visual difference. Compare paired empty-field
screens at identical density, theme and font scale: flat control interiors must differ by
at most 3/255 per RGB channel, with a one-physical-pixel edge band excluded for raster
anti-aliasing; measure control bounds separately. For full pictures, mask system bars and
one-pixel glyph edge bands, count every remaining channel error above 3/255, and report
mismatches rather than approving a percentage of different pixels. Keep that method across
light, dark and font 2.0. Wrapping titles/summaries where Compose uses ellipses is an explicit
accessibility improvement and is excluded from parity failure; all text sizes and line
heights otherwise use Android's nonlinear system `sp` conversion, exactly as Compose does.
Selection handles must work, are drawn in Rust as M3 primary-colour teardrops, and may use
Android's floating ActionMode for the toolbar. The settings root yields Back to the system
animation; in-app pops receive gesture progress from Android's OnBackAnimationCallback
(via AndroidX) and Rust applies it to the prototype scene. Native-load failure uses a minimal
platform View crash adapter before preferences/native UI bootstrap. It has a plain-language
summary and Copy details button, with a reusable structured report boundary so it can host
all crashes after Compose deletion. Details exclude exception messages/causes, filenames
and all editable/clipboard/transcript/learned/audio data. **Would change if** phone evidence
requires a measured, explicitly agreed platform exception; the full port and fallback
deletion remain separately gated.

## Suggestions take the whole Smartbar row while words show (2026-11-15; superseded 2026-11-16 by Gboard's row, below)

On a 384 dp phone the shared row gave three suggestions about 250 px each between the mic,
the speaker, the emoji and the arrow, so any word longer than five letters was cut off. Now,
in the shared and extended layouts, the row's actions step aside whenever there are words to
show, flipped or not, and the words get the whole width. With no words, the row is exactly
as before, and so are suggestions-only, actions-only, inline autofill and the opened action
row. Two exceptions keep it as before: dictation (running, or its preview showing), because
its controls must stay under the finger, and read-aloud while it plays with the speaker
shown, because the speaker is the stop button. `Config::roomy` in the render crate is the
one rule. Compose asks it over JNI, and touch and TalkBack follow the same layout. Every
keystroke empties the suggestion list for a moment before the new words arrive, which would
make the icons blink in and out while typing. So the NLP session's `words` fact holds across
that gap, and only across that gap: an empty refill, incognito, a selection or a clear all
end it. A word still too wide shrinks in whole-percent steps down to 75% before it gets an
ellipsis, measured by each renderer's own text engine with the scale from
`smartbar::text_scale`. **Would change if** using the phone shows the mic or emoji are missed
while typing; then keep just that one action beside the words, not the old four.

## Words show in Gboard's row, mirrored for a left hand (2026-11-16)

Giving the words the whole row hid the mic exactly when you are typing, and the row no longer
looked like Gboard's, the reference keyboard. So while words show in the shared or extended
layout, the row is now Gboard's: the sticky action (the mic), the three words, and the
expand arrow. The user is left-handed, so with flipToggles on (the default) the mic is on the
left and the arrow on the right; with it off the row mirrors. With no sticky action the mic's
slot is absent, not left empty, and the speaker steps out while words show. The ends are
Gboard's sizes. In Gboard 15.1 at 420 dpi (411 dp wide), the mic's slot measures about 40 dp
(a 34 dp circle) and the features button about 30 dp, leaving about 114 dp per word. Our
Smartbar row is 40 dp at the default keyboard height (`row * 0.753`, 150 px on the phone), so
the mic gets a row-high square with no padding and the arrow three quarters of the row
(`GBOARD_TOGGLE = 0.75`, rounded to the pixel). Both scale with the row if the keyboard
height changes. The arrow keeps its 40 dp button, drawn centred on the narrower slot, so only
the button's transparent 6 dp margins overhang it and its 28 dp circle still fits inside. Touch
and TalkBack keep the slot itself. On the phone (1440 px at density 3.75, 384 dp) that is a
mic of 150 px (40 dp) and an arrow of 113 px (30.1 dp). That leaves 1177 px for the words, and
after two 1 dp (4 px) gaps, 389, 390 and 390 px each (103.7 to 104 dp). Inside each word's 4 dp
margins and 8 dp padding, 300 px (80 dp) holds the text: at 14 sp (52.5 px) "the" needs 73 px,
"wonderful" 231 px and "unfortunately" 308 px (Roboto Light, host measure), so
"unfortunately" squeezes to 0.97 and all three show whole. A long word fits the way AOSP
LatinIME's suggestion strip (the base for Gboard's) does it. First it squeezes: its glyphs get
narrower at the same height (text scale X), in whole percents rounded down, down to 0.70. That
floor is LatinIME's `MIN_TEXT_XSCALE = 0.70f`
(`java/src/com/android/inputmethod/latin/suggestions/SuggestionStripLayoutHelper.java`,
line 67, platform/packages/inputmethods/LatinIME `main` at 127336e9f29d, read 2026-11-16).
Only then does the font shrink, down to the existing 0.75, and only past both does the word
ellipsise. We keep our end ellipsis, where LatinIME's goes in the middle. On the phone a word
squeezes until it needs 428 px and ellipsises only past about 571 px. `smartbar_fit::fit_step`
is the one rule, and each renderer gives it its own text measure: Compose draws the result
with `TextGeometricTransform(scaleX)` and Rust with a non-uniform glyph transform. These are
unchanged: dictation and its preview, read-aloud with the speaker shown, the NLP hold across
a keystroke's refill, suggestions-only, actions-only, inline autofill, the opened shared row,
the extended second row, and the row with no words (minus the emoji, below). **Would change
if** a mic on the left proves wrong for typing in use; then flip it back with flipToggles, or,
if Gboard's sizes change, re-measure them.

## Emoji is the key beside the space bar (2026-11-16)

The Smartbar's pinned emoji button took about 48 dp of the row and had nowhere in Gboard's row
(above). Gboard keeps emoji beside the space bar instead, so we do too. The utility key now
defaults to on, with the action `SWITCH_TO_EMOJIS` (`catalogue.tsv`). Every characters layout
(default, Dvorak, Dvorak DE and SE, Neo 2) already has it right before the space bar as
`IME_UI_MODE_MEDIA`, and that action keeps the language-switch key beside it hidden, so the
bottom row gains just the emoji key, which takes its width from the space bar. The pinned emoji (`PinnedEmojiAction`, Rust action 2) is gone from the main row
in every state, in both renderers. Slot 2 stays reserved in the packet so the ids shared by
the host, touch and TalkBack do not move. A setting the user has already saved overrides the
new default, as with any jetpref value. **Would change if** a second language is installed and
the language switch is needed on that key; then put emoji on the key's long press instead.

## Word statistics from a public corpus (2026-11-16)

The English dictionary's frequencies and next words are counted from public text we can name and
rebuild, not taken as AOSP's dictionary had them. AOSP's file came from text nobody can see, held
at most three next words a word, and gave next-word top-1 of 4.8% on held-out English
([results](language-model-results.md)).

**Corpus.** English sentences from two OPUS collections, pinned by version and SHA-256 in
`scripts/build-language-model.sh`, never committed:

- [Tatoeba v2023-04-12](https://opus.nlpl.eu/Tatoeba/corpus/version/Tatoeba): short everyday
  sentences by volunteers, [CC BY 2.0 FR](https://tatoeba.org/en/terms_of_use) (terms of use
  §6.2). The most conversational licence-clear English there is, which is what people type.
- [GlobalVoices v2018q4](https://opus.nlpl.eu/GlobalVoices/corpus/version/GlobalVoices):
  citizen-media articles, [CC BY 3.0](https://globalvoices.org/about/global-voices-attribution-policy/).
  General written English, names and topics, so the statistics don't only know small talk.

Both allow use with attribution, commercial use included. No sentence ships, only counts merged
into AOSP's word list; the attribution and the change are in the app's third-party licences
screen (`app/src/main/config/`, which AboutLibraries lists) and in the results doc. OPUS asks to
be cited as J. Tiedemann, 2012, *Parallel Data, Tools and Interfaces in OPUS*, LREC 2012.
Rejected: Wikipedia and OpenSubtitles (share-alike or unclear licences for a shipped derivative),
Common Crawl and Reddit dumps (no licence to the text), and books corpora (old English, not
typing).

**Dictionary.** The same v202 format, written by our own writer (`crates/lexicon`), so the
decoder, its C++ oracle and the APK's memory-mapped asset are unchanged. AOSP's words, flags and
shortcuts are kept: its possibly-offensive and not-a-word marks were made by hand and the corpus
can't remake them, so those entries keep their values and are never a next word. Unigram values
are AOSP's own set of values handed out again by corpus frequency, keeping the scale the decoder
was tuned on. Up to 32 next words a word, each seen at least twice: 2,370,665 bytes against
AOSP's 2,956,881, so the APK is 585,600 bytes smaller. 16 save fewer keystrokes; 64 save a few
more but correct more clean words and swipe a little worse; counting pairs seen once adds 1.7 MB
for 0.1 point.

**Model.** On top of the dictionary, a word trigram model from the same counts and over the same
words (`main_en-US.lm`, `crates/ngram`, built by `lexicon model`): interpolated Kneser-Ney, pairs
and triples kept if seen at least three times, each log probability and backoff a byte. 77,757
words (the dictionary's words the corpus writes, less the flagged ones, so it can never predict
them), 712,166 pairs, 1,081,773 triples: 12,188,272 bytes, stored uncompressed and read from the
APK into native memory (12.2 MB; nothing on the JVM heap). It predicts the next word from the
last two, re-weighs typed candidates in context (three quarters of its lift on the dictionary's
scale) and tells the swipe ranker which guesses the context favours; the user's learned next
words still come first. Next-word top-1 goes from 13.3% to 19.0% and top-3 from 21.0% to 30.7%,
keystroke savings from 28.9% to 36.2%, with correction as good or better on every measure and
swipe top-1 from 91.2% to 91.9%, for 0.05 ms more per keystroke
([results](language-model-results.md#stage-c-a-trigram-model-2026-11-16)).

- *Why trigrams, not bigrams:* the same pairs without triples (5.7 MB) predict little better
  than the dictionary (top-1 13.6%) and swipe worse; the word before last is what pays.
- *Why not 4-grams:* the budget. Triples seen twice are already 16.8 MB; triples seen three times
  leave 2.8 MB for 4-grams, which 31 million words would fill with few that recur. Pruning harder
  to make room costs what 4-grams might add (seen four times, top-3 falls 0.3 point).
- *Why these thresholds:* keeping pairs seen twice (14.1 MB) adds 0.3 point of savings and at
  most 0.1 elsewhere, for 1.9 MB more APK and memory. Pairs or triples seen four times save
  1.0–2.8 MB and lose 0.1–0.3 point.
- *Why stupid backoff isn't used:* Kneser-Ney's probabilities add up, so the lift on the
  dictionary's scale means the same in every context; the reader's cost is the same.
- *Neural model:* not built. onnxruntime is already in the APK, but the numbers say little is
  left for it to win. Correction and swipe hardly depend on the language model any more: the
  whole trigram model moved them by at most 0.7 point, so a better one could only help next words
  and savings. There, a neural model's edge over counts is longer context and sharing what it
  learns between similar words, and it would be trained on the same 32 million licence-clear
  words, small for one; its gain here is unmeasured, and it would cost megabytes and an inference
  per keystroke against the n-gram's 0.05 ms. Worth measuring only with a larger licence-clear
  conversational corpus, and then first for next words alone (once a word, after the space).

**Would change if** a larger licence-clear conversational corpus appears (more next words and
4-grams would then pay, and a neural model could be trained on it), if a second language needs a
dictionary (the same build, with its own corpus and AOSP's word list for it), if the APK or the
keyboard's memory gets tighter (pairs and triples seen four times give 9.3 MB for 0.3 point), or if
harmful corrections of unknown names grow with the user's own writing; then the auto-correction
threshold, not the statistics, is the place to fix it.

## Learning where the user taps (2026-11-16)

**Decision.** The keyboard learns, per key and per orientation, where this user's taps land
in the key: a running mean of their tap offsets (in key widths and heights, as `TapOffset`
measures them) and the spread about it (`crates/suggest/src/tap_aim.rs`). It uses that in two
places: **(a)** the decoder hears each tap moved back by its key's learned bias, so a tap where the
user usually hits P reads as a centred P; and **(b)** the hit test moves a tap near a border to
the neighbour the user meant, but only when both keys' learned aims agree. Both are on by default
behind *Learn how I tap* (Typing), which incognito, password, e-mail and URL fields override like
learning words. The model is `files/tap-aim.txt`, numbers only (a key's code point and its
statistics), left out of backups like `learned-words.txt`, and *Forget learned words* erases it.

**What it learns from.** Only words committed as typed and kept (`crates/suggest/src/tap_learning.rs`):

- finished by a space, punctuation or Enter with no auto-correction, in a field where words are
  learned (the NLP session's "aim" rule: the setting, a corrected-prose field, not incognito, not
  a raw editor);
- with no backspace inside the word: every letter typed since the last deletion, so a word fixed
  in place teaches nothing;
- only the characters that were tapped (a long-press popup's letter has no position);
- and only once nothing takes it back. The word waits until two more characters are typed, the
  next word is kept, a prediction is tapped or a word is swiped. A backspace, the undo key, a
  suggestion that replaces the word, or a new field throws it away.

Auto-corrected words the user kept do not teach. Measured: their letters add at most 0.12 point,
and at most 0.18 point even with each tap re-measured from the corrected key's centre. And a wrong
correction the user missed would teach the wrong key.

**Safety limits** (`tap_aim.rs`):

- A key applies nothing until it has 16 samples.
- One standard error of the mean is taken off as noise, and what is left applies only from 0.03
  key.
- A shift is capped at 0.25 key. A sample counts at most 0.75 key from the centre.
- The mean follows a key's last 200 samples. Every sample fades every other key, so a key the user
  stops hitting drops back below 16 samples and stops applying.
- Portrait and landscape are learned apart, because the grip and the keys' shape change with
  them. A key is learned under its lower case, and at most 64 keys are kept per orientation.

**(a) against (b).** On the scoreboard's learning run, the left-thumb user's word accuracy after
autocorrect goes from 93.67% with learning off to 93.88% with (a) alone, and 94.84% with (a) and
(b). (b) is most of the gain because it stops typos rather than repairing them: typos made fall
from 37.8% to 33.0%, while (a) only helps the decoder guess (missed typos 14.2% → 13.7%). The
neutral typist is exactly unchanged by both: 96,795 of 99,893 words right in all three columns,
the same 79,445 words tapped cleanly. Without the 0.03 floor the neutral typist lost 2 words with
(a) and 3 with (a) and (b), 2 of them tapped wrong; with it, nothing. A user whose grip changes halfway is better
from the first 500 sentences after the change (94.20% against 93.61% off), and by the rest matches
a user who always had the new grip (94.02% against 94.00%), so the decay works. The full tables:
[tap-aim-results.md](tap-aim-results.md).

- *Why (b) ships though the brief said "conservative":* it is conservative where it counts. It
  moves a tap only where both keys' aims say so, never for a key without 16 samples, and never
  by more than a quarter key. It measurably never hurt the neutral typist, and it is 80% of the
  gain.
- *Why not learn from corrected words:* see above. It is little gain, for a risk the scoreboard
  can't see.
- *Why per key, not one offset for the whole hand:* a thumb's error differs across the keyboard
  (short on the far side, low near the bottom). The scoreboard's users with their own aim per key
  gain as much as the plain left thumb (93.11% → 94.50%).

**Would change if** the phone's counters (`files/typing-counters.txt`, see
[rust.md](rust.md#the-typing-counters)) show more typos left or more corrections undone with it on
than off, if users with very different grips share a phone (the window and fade would then need a
per-user switch), or if a layout with keys much smaller than QWERTY's needs a smaller cap.

## Swipes decoded beyond FlorisBoard's classifier, checked on two simulators (2026-11-16)

The swipe scorer keeps FlorisBoard's classifier exactly (`glide::Tuning::ORIGINAL`, the flat
frequency curve), which the Kotlin reference tests still hold it to, and the keyboard decodes
with four cheap additions on top: one frequency slope all the way up, the swipe's ends weighed
against the word's end keys, its slow stretches counted for more (the renderer now sends each
glide point's time), and a word held to the swipe's reach. With a learned word boosted the more
the user uses it, that takes swipe top-1 on the report text from 91.9% to 93.9% with a neutral
hand and from 87.5% to 91.1% on the second simulator, top-3 up on all four variants, for about
0.1 ms more per swipe ([results](swipe-accuracy.md)).

- *Why a second simulator:* every change was tuned against a simulated finger, and a change can
  learn that finger instead of a person's. The scoreboard's own swipe is a path by rule; the second
  is the sigma-lognormal model of human movement (Plamondon 1995; Plamondon & Djioua 2006), as
  Leiva, Martín-Albo & Plamondon synthesise gestures with it (2015). It slows at every letter and
  cuts corners by stroke overlap, and was frozen before any change was measured. It earned its
  place: a strong reach that kept paying on the first cost the second up to 2 points.
- *Why the keep rule:* top-1 up on both simulators, with a neutral hand and a left thumb, and top-3
  down on neither, chosen on sentences the report doesn't use. Each change passed alone, the
  boost on top of the curve.
- *Why the boost only with the curve:* on FlorisBoard's flat top, a boosted "three" or "too" ties
  "the" and "to" and the boost costs about 0.8 point of top-1 everywhere. On one slope it gains
  0.02–0.4 point; small on the rule-drawn finger, and measured with a long in-domain history,
  so a real user's gain depends on their typing. It is capped (20 steps from 16 uses) and goes
  with the learned words.
- *Why not double letters or subset paths:* neither leaves a mark a swipe can show ("hello",
  "to"/"too"), so nothing was measurable. The curve, reach and the ranker's words-before already
  separate them as far as anything can.
- *Dwell at 1.5, not 2:* 2 was at most 0.09 point better on the tuning text; the gains flatten,
  and a stronger weight trusts the simulators' speed profiles more than a real finger may deserve.

**Would change if** real swipes can be measured without logging them and show a change helps
less than both simulators say; then re-tune on those, starting with dwell and reach, which depend
most on how a finger moves. A better language model would shift the boost's and the
curve's balance against the ranker and should be re-measured with them.

## Auto-correction strength: Off, Modest, Normal, Aggressive (2026-11-16)

*Typing → Corrections → Auto-correction* sets how readily typing is auto-corrected. Normal, the
default, is the 0.185 threshold the keyboard always used (AOSP calls that value "modest"), so
nothing changes unless the user picks another. Modest is 0.25, Aggressive 0.12, and Off never
auto-corrects, not even the whitelist, while suggestions still show
([results](autocorrect-results.md#auto-correction-strength)).

- *Why these thresholds:* chosen on sentences the report doesn't use. Modest is the lowest that
  halves harmful corrections for both a neutral hand and a left thumb; Aggressive is the highest
  at which a left thumb's word accuracy peaks (93.7% → 94.3% on the report text, harmful 0.5% →
  0.9%). AOSP's aggressive 0.067 is worse than 0.12 on every row.
- *Why Normal stays the default:* it is still the best neutral accuracy, and it is what the user
  has been typing with. Aggressive is the better choice for this user's left thumb by 0.6 point at
  the price of about one more changed word in 250; that is a preference, so it is offered, not
  imposed.
- *Why the names:* Gboard has only on/off; HeliBoard's "modest/aggressive/very aggressive" would
  call today's behaviour "modest", which reads as the weak end. Normal for what it always did keeps
  the list honest.
- *Where it lives:* the thresholds and the rule are Rust's (`suggest::auto_correction::Strength`);
  Kotlin reads the preference and passes the strength's ordinal, which a test holds to Rust's order
  and to the settings catalogue. The spell checker keeps Normal's confidence whatever the setting.

**Would change if** the phone's typing counters show corrections undone at a rate that points to a
different default for this user, or a better language model moves the best threshold (re-run the
sweep in [autocorrect-results.md](autocorrect-results.md)); a strength would then move, not the
default's meaning.

## Backspace undoes an auto-correction, however fast it is pressed (2026-11-16)

One backspace straight after an auto-correction puts back the word as typed, as Gboard does, and
the word is then kept as typed: it is not corrected again (for the session only, in incognito).
Most of this already worked and is now held by tests: a single press, the punctuation that set the
correction off kept in place ("teh," stays "teh,"), the space auto-space punctuation added after it,
a word typed after an automatic space, the undo read from the editor's predicted text when the
backspace comes before the app's cursor updates, and the Rust renderer's keys going through the
same dispatcher. Any other key ends the chance to undo before it acts; Ctrl+backspace and forward
delete never undo.

One case was broken: typing on fast. Undoing "teh " → "the " leaves "teh" owing a space, so the
next letter starts a new word ("teh y"). The app reports each edit's cursor separately, and when
those reports arrived after the undo, each one ended the owed space, giving "tehy". Now the editor
session (`editor` crate, its `late` call) recognises a report of one of its own edits that it has
already followed with more, and such news of the past no longer ends the spacing the later edits
set.

- *Where it lives:* the judgement is Rust's: whether an update is late compares it against the
  edits the session expects to be reported, the same queue that already decides which reports are
  the keyboard's own. Kotlin asks and, if late, skips ending the spacing (`followUpdate`).
- *Why not move the rest of the undo into Rust:* where to revert (`undo::revert_at`) and the
  delete plan's first step already are; the remaining Kotlin is the edit itself and the stores.
- *Proof:* Rust tests in `editor`, `input` (delete plan, routing order); five flows through the
  native editor and a fake app in `AutoCorrectionUndoFlowTest`, including cursor reports delayed
  past the undo; and four mutations in `scripts/correction-mutations.py`, each caught.
- *The Rust settings try-out field (follow-up, same day):* its `SettingsInputView` answered
  `beginBatchEdit`/`endBatchEdit` without doing anything, and `RustSettingsView` reported the
  cursor after every call. So the keyboard heard each step of its own batch ("the" committed, then
  the space), steps it doesn't expect, which it reads as the user moving the cursor: the undo still
  put back "teh", but the space it owes was ended and "y" gave "tehy". Now the settings editor
  counts the open batches (`render::settings_ui::editor::Batches`, on the settings `App`) and
  says in its state whether to report (`"report"`). Like EditText, it reports once the outermost
  batch ends, `endBatchEdit` answers whether one is still open, and a closed connection ends the
  batches it left open. `AutoCorrectionUndoFlowTest` runs the same flow against the native
  settings editor through an adapter that maps calls as `SettingsInputView` does
  (`SettingsFieldConnection`); before the change it fails on the owed space. Three more
  mutations, each caught. On the emulator's try-out field: "teh " became "the ", one backspace
  gave "teh" and "y" gave "teh y".

**Would change if** the owed space should go (the next letter continuing the word instead), which
would be a change to `undo::revert_at`, or the phone's typing counters show undone corrections
being re-corrected; the late-update rule would stay either way. The settings field's batches
would change only if Android's EditText stopped deferring its cursor reports, the behaviour it
copies.

## Apostrophes: the language model decides between "its" and "it's" (2026-11-16)

Typing a contraction without its apostrophe already worked where the letters spell no word
("dont", "im", "youre", "cant"): AOSP's decoder and whitelist put it right, as Gboard does. Where
they spell a word too, AOSP decided by kind, not by meaning: a valid word was never corrected ("I
think its" stayed), a whitelisted one always was ("he is ill" became "he is I'll"). Now the bundled
n-gram model decides those from the words before, when it finds one spelling at least 1.5 nats
likelier than the other; when it isn't that sure, or lacks a word, AOSP's choice stands
([results](autocorrect-results.md#apostrophes)).

- *Why the model and not a list:* "ill", "lets", "its", "were", "well", "shed", "hell" are each
  right in some sentences and wrong in others, which no word list can know; the model can, and it
  is already loaded and already re-weighing candidates. For a typist who drops apostrophes it puts
  back 85.6% of them instead of 73.5% (2,527 of 2,953 instead of 2,170).
- *Why 1.5 nats:* the knee of the tuning sweep; it costs 10 harmful corrections in 121,594 clean
  words for that typist, and 6 in 99,461 for one who types apostrophes, mostly plurals read as
  possessives.
- *Where it lives:* `suggest::apostrophes` (the rule and margin) and the suggester's
  auto-correction decision; the session answers the model's odds (`Dictionary::ln_odds`). Without
  a model the trait answers none, so AOSP's behaviour, and every reference-parity test, is
  unchanged. Nothing in Kotlin changed.
- *Proof:* unit tests over a fake model (kept, restored, unsure, capitals, mid-word, Off), tests on
  the shipped dictionary and model in context, the scoreboard's new typist, and six mutations in
  `scripts/correction-mutations.py`, each caught.

*Requiring an n-gram (tried 2026-11-16, not shipped):* "we wed" → "we'd" and "my id" → "I'd" happen
because the model has seen neither spelling after "we" or "my", so its odds are word frequency
alone. Putting an apostrophe back only where the model has seen it after the words before keeps
both, but on held-out text it puts back 2,498 instead of 2,527 apostrophes to avoid 4 harmful
corrections in 121,594 words: frequency alone is right about ten times as often as it is wrong
(possessives such as "Toms", "Obamas"; "cant", "wont"), and nothing the model knows separates
"we wed" from "in the end wont" ([results](autocorrect-results.md#requiring-an-n-gram-to-put-an-apostrophe-back-tried-not-shipped)).
On the emulator (Rust renderer and settings) the shipped rule gives "we we'd", "my I'd" and
"don't" for "we wed ", "my id " and "dont ", as measured.

**Would change if** a better model or a bigger corpus moves the knee (re-run the sweep), the
phone's counters show restored apostrophes being undone, or possessives are wanted at a different
risk. "We wed" and "my id" would want a model that knows word classes (a pronoun is not followed
by a pronoun's contraction); requiring an n-gram is measured above and costs more than it saves.

## Field kinds: URL, e-mail, search, filter and password fields (2026-11-16)

Kotlin already handed Rust the field's raw `inputType` and `imeOptions` (`NlpEnvironment.fields`)
and the key variation Rust reads from them (`EditorAttributes.keyboard`); the rules for each kind
of field now all live in Rust, and Kotlin's wiring is unchanged.

| Field | Suggestions | Auto-correction | Keyboard's own spaces | Learning, kept words |
|---|---|---|---|---|
| Text, search box (action Search) | yes | yes | yes | yes |
| Filter (`TYPE_TEXT_VARIATION_FILTER`) | yes | **no** (was yes) | yes | **no** (was yes) |
| URL, e-mail, web e-mail | yes | no | **no** (was after a pick or swipe) | no; kept words **not remembered** (were) |
| The same, asking for auto-correction (`TYPE_TEXT_FLAG_AUTO_CORRECT`) | yes | **yes** | after a pick or swipe, **not** after punctuation | no |
| Password (text, visible, web, number) | no | no | no (nothing to pick or swipe) | no |
| No-suggestions flag | no | no | yes | no |

And in ordinary text, **nothing of an e-mail address or URL is learned**: a word glued to an '@',
'/' or ':', or to a '.' between letters ("example." before "com"), is not, and the piece before it
in the same address, learned before anything showed it was one ("example" when the '.' was typed),
is taken back. A whole address was already never stored (a learned word is letters only).

- *What was already right:* no auto-correction or learning in URL and e-mail fields
  (`nlp::policy`), no suggestions in passwords, no auto-space after punctuation outside ordinary
  text. *What wasn't:* the phantom space a picked or swiped word owes was still typed in URL,
  e-mail and password fields ("hello" swiped, then "world": "hello world"); a word kept as typed
  was remembered whatever the field; URL paths and domain labels typed in prose were learned.
- *Search, checked against Gboard on the emulator:* in Settings' search box (`0x100001`, action
  Search) Gboard corrected "teh" and "wprld" and put a space after a picked word, so this keyboard
  treats it as text, as it already did. In a filter field (Private DNS hostname, `0x200b1`) Gboard
  left "wprld" alone, so filter fields are now not corrected (AOSP hides suggestions there).
- *E-mail, against Gboard:* the user's rule (no correction, no auto-space in URL and e-mail fields)
  is followed for a field that doesn't ask. In Contacts' e-mail field, which asks for
  auto-correction (`0x8021`), Gboard corrected "wprld" and spaced after a pick, so since the later
  2026-11-16 follow-up a URL or e-mail field that sets `TYPE_TEXT_FLAG_AUTO_CORRECT` is corrected
  and a word picked or swiped there is spaced (`nlp::policy::Context::autocorrect`,
  `spacing::spaces::spaces_words`). Still, nothing typed there is learned or remembered as kept,
  and tap aim isn't learned from it (`Context::prose`): an address is not the user's vocabulary,
  whatever the field asks. Punctuation is not auto-spaced there either ("example. com" would break
  the address). A filter or password field is never corrected, whatever it asks. A URL field
  wasn't checked against Gboard: the only one on the emulator is Chrome's, behind its terms of
  service; it follows the e-mail field, as both are addresses.
- *Where it lives:* `nlp::policy` (`NOT_PROSE`, `ADDRESSES`, `prose`, `autocorrect`,
  `remember_kept`), `spacing::spaces` (`phantom_space_before` asks `spaces_words`: the key
  variation, as AOSP's `isAutoSpaceFriendlyType`, and for an address the field's raw `inputType`,
  which the editor session already had from its `config` op),
  `suggest::addresses` and the word learner. The frozen Kotlin references (`ReferenceNlp`,
  `ReferenceSpacing`, `ReferenceLatin`'s word learner, `NlpLearningMatchTest`) carry the same
  changes, noted in their headers.
- *Proof:* Rust tests per rule, `FieldKindFlowTest` from the raw `EditorInfo` values through the
  native editor and NLP session, and sixteen mutations in `scripts/correction-mutations.py` (eight
  more for the fields that ask for correction: the clause itself, the flag check, learning, tap
  aim, both spacing directions, and the input type dropped by the editor session or the JNI
  bridge), each caught. On the emulator, in Contacts' e-mail field (0x8021): a picked "the" was
  spaced before the next letter, "adn " became "and ", one backspace undid it, and the learned
  and kept word files were unchanged afterwards.

**Would change if** a URL field on Gboard turns out not to correct or space when it asks for
correction, the user wants addresses typed in such a field learned, or prose writers miss words
after "and/" or "e.g." (the address test is deliberately broad: a word it wrongly skips is simply
not learned this once).

## Keys resized by the letter odds (2026-11-16)

A tap near the border between two letter keys types the one likelier to come next in the word,
by the n-gram model's odds for the words before and the letters typed: the border moves towards
the less likely key by 0.04 × the log odds key widths across (0.025 between rows), at most 0.15
key, so a tap further inside its key than that always types it. It ships, always on, in both
hit tests (`keys::resize`), and costs about 8 µs a tap on the host with the odds worked out at the
tap (the phone works them out on the suggest path). On held-out text it cuts typos made from 38%
to 20% for a left thumb and from 20% to 8.5% for a neutral hand; word accuracy after autocorrect
goes from 93.72% to 95.18% (left) and 96.92% to 97.62% (neutral), with harmful corrections down
on every hand ([results](key-resize-results.md)). With tap-aim learning on as well, the left
thumb goes from 94.84% to 95.85%.

- *How it combines with the learned aim:* the aim goes first and the odds never undo it. Where
  the aim keeps a tap, the odds act on where the user meant it, stopped at the key's border.
  The aim is this user's own evidence about their finger; the odds are a guess about the text.
- *Why weaker than the theory:* Bayes' rule with the simulator's own tap spread gives 0.0625 and
  0.04. That is 0.04–0.16 point more accurate on the report text but loses about 40% more words
  the plain hit test had right, and a real finger isn't the simulator's. The factor is the
  simulated spread scaled to 0.2 key across.
- *Why the rare words are safe enough:* words outside the dictionary are typed right more often
  with it on every hand (64.42% → 65.65% neutral, 46.75% → 49.30% left), and more of them are
  committed right (3,149 → 3,209 and 2,285 → 2,410). The odds turn themselves off as soon as no
  known word starts with what was typed. Their harmful-correction *rate* rises (14.45% → 14.88%
  neutral) only because more rare words are now tapped cleanly, which autocorrect then sometimes
  changes; counted by word they gain (175 won against 115 lost, neutral).
- *Why the odds are checked against the tap trail:* suggesting runs off the hit test's path and
  can be a keystroke behind. A key going down uses the odds only while the letters typed end in
  the word they were worked out for and no other key is down; for a first letter, only straight
  after the space bar. Rollover therefore gets no resize (the odds would be a letter behind), nor
  does the first key after a backspace or shift press. Two stale cases remain: a word that ends
  in its own prefix ("t" then another "t") and an empty-word deposit from an earlier word boundary;
  both only move a tap within 0.15 key of a border. *Looked at again the same day and left:* no
  clean fix lives in Rust alone. The trail sees neither spaces as text nor cursor moves, so a
  letter count since the space bar fixes only the first and loses the odds after a cursor tap;
  an exact fix needs the odds tagged with the content they were worked out for, carried from
  Kotlin's suggest request or published by the native editor session
  ([results](key-resize-results.md#the-rule)).
- *Tried and dropped:* backing the prior off as the typed prefix gets unlikely (no better on rare
  words, worse overall), leaving first letters alone (0.58 point worse for the left thumb), caps of
  0.2 and above (a little better overall, worse for the neutral hand's rare words), and floors of
  0.05 and 0.2 (no clear gain over 0.1).

The fast-typing check found that a letter tapped over a held space was typed before the space
("thec " for "the c"), and one tapped over a held delete was deleted by it: both keys act when
they go up, and neither renderer let them go when another finger landed. Now a still space or
delete goes up, typed, when another finger goes down (`input::pointer::rolls`), as letters
already did; one that has swiped or been held long stays, so a cursor swipe goes on and a hold
keeps its action. Shift stays down, so shift-and-tap still capitalises.

**Would change if** the phone's counters ([rust.md](rust.md#the-typing-counters)) show more
backspaces or undone corrections with it than without; if the suggest request ever carries an id
to the JNI call for another reason (then tag the odds with it and close both stale cases); if a
layout's keys are much smaller than QWERTY's (the factor is in key widths, so it scales, but the cap may need to shrink); if a
better language model is shipped (re-tune the factor against it); or if real taps can be measured
without logging them and spread wider or narrower than 0.2 key, which sets the factor.

## The emulator's settings baseline restores itself (2026-11-16)

`scripts/settings-device-baseline.py` had a capture mode and `--verify`, which only compares,
so nothing but `SettingsDevice.cleanup` inside a device-test run put the AVD back, and a run
that died before its cleanup left the AVD in its fixture state (follow-up 3). It now has
`--restore`: put back whatever differs (the Android settings, dark mode and log tags, then the
app's files while its process is dead), bind the original keyboard, mute every stream, then
check exactly as `--verify` does. `--verify` stays a check that changes nothing, rather than
restoring first, because "passed before anything changed" is what a run's receipt says about
the state it found; a check that restored first could never fail and would hide a run that left
the AVD dirty. A mismatch now names every differing part, still never a value. The device-test
runner's exit trap restores. Tried on aw_keyboard_ssd the same day, after the polish checks:
`--verify` failed naming `secure, files` (agent-device's keyboard had been enabled, and the app's
files had changed), `--restore` put both back and checked them in about 3 s, a fresh `--verify`
passed, and agent-device's keyboard was no longer enabled. **Would change if** the baseline gains a part the
shell can't set back (then `--restore` must say so rather than pass), or a harness that owns the
AVD's state replaces these scripts.
