# Language model results

How well the keyboard predicts, completes, corrects and swipes English, from the typing
scoreboard (`crates/scoreboard`; [how to run it](rust.md#the-typing-scoreboard)). Every number
here is from the same held-out text, seed and simulated phone, so rows compare directly.

## The test

**Text.** 10,000 held-out sentences: the first 5,000 (in hash order, so a uniform sample) of each
source's held-out set, 124,547 scored words.

| Source | What it is | Licence | Why |
|---|---|---|---|
| `tatoeba` | [OPUS Tatoeba](https://opus.nlpl.eu/Tatoeba/corpus/version/Tatoeba) v2023-04-12, English: short everyday sentences written by volunteers | [CC BY 2.0 FR](https://tatoeba.org/en/terms_of_use) (terms of use §6.2) | The most conversational licence-clear English there is: questions, replies, plans, feelings |
| `globalvoices` | [OPUS GlobalVoices](https://opus.nlpl.eu/GlobalVoices/corpus/version/GlobalVoices) v2018q4, English: citizen-media blog posts | [CC BY 3.0](https://globalvoices.org/about/global-voices-attribution-policy/) | General written English, names and current topics, to keep the model from only knowing small talk |

The corpus is split once by a hash of each sentence's words (lower case, letters and digits
only), one in a hundred held out, and only the first copy of each sentence is kept on either side.
So no held-out sentence is in the text the new dictionary and model are built from; sentences
that differ by a word or two can still fall on both sides, which Tatoeba has many of ("Tom is
tall." / "Tom is tired."). The shipped AOSP dictionary was built by Google from text we can't see,
so it may have seen some of these sentences; nothing here could favour it for that.

The scored words are letter words: letters, with apostrophes or hyphens inside ("don't",
"e-mail"). Numbers and anything touching a digit are typed as context but not scored. Comparisons
ignore case, since the keyboard capitalises a strip to match the sentence.

**The keyboard.** The shipped en-US QWERTY layout and default settings at a Galaxy S24 Ultra's
portrait size: 1440 px wide at 3.75 px per dp (384 dp), a 199.2 px base row, keys 142.5 px apart
across and 199.2 px down, laid out by the keyboard's own `keys` crate. Nothing learned: no
personal dictionary, no learned words.

**Metrics.**

- *Next word*: after the space before each word, whether the strip's first candidate, or one of
  its three, is the word.
- *Keystroke savings*: the user types each word letter by letter and taps it in the strip as soon
  as it is shown (after the space or after any prefix), counting that tap. Savings are the share
  of letters not typed. Spaces don't count on either side.
- *Auto-correction*: each letter is tapped with Gaussian scatter around the key's visible centre
  (a quarter of the key pitch across, 36 px or about 1.8 mm; a fifth of the row pitch down,
  40 px). The keyboard finds the key under the tap by its touch bounds and the tap's offset in it,
  as on the phone, and the decoder gets both. Then the space bar commits the auto-correction if
  there is one (`autoCommitCandidateFor`). A tap that lands off the letters (space, shift,
  delete, comma) is drawn again: those are different mistakes. Characters with no key of their
  own (accents, apostrophes) are typed as they are.
  - *Neutral*: as above.
  - *Left thumb*: one-handed with the left thumb, pivoting at the keyboard's bottom left: the
    farther the key, the more the tap falls short (up to 0.15 of a key left and 0.1 of a row
    down at the far top right) and scatters (up to 15% more).
  - *Word accuracy*: the committed word is the word meant. *Harmful*: a cleanly typed word was
    changed (share of clean words). *Missed*: a typo was left as typed (share of typos). *Wrong
    fix*: a typo was changed into another wrong word (share of typos).
- *Swipe*: a path aims at each of the word's keys (a doubled letter once), missing the centre by
  a Gaussian of a fifth of the key, cuts each corner by up to a third of a key, is drawn as a
  smooth curve with a little tremor, and keeps points as the keyboard does (a quarter of a key
  apart). The eight best words are re-ranked by the words before them; the first is committed.
  Top-1 is the committed word; top-3 adds the strip's next two. Words needing fewer than two keys
  ("I", "a") aren't swiped. This is the `trace` simulator with a neutral hand; the scoreboard now
  also swipes with a left thumb and with a second, published model of movement, and the keyboard
  decodes swipes better than in the rows here ([swipe accuracy](swipe-accuracy.md)).
- *Strip time*: how long one strip takes while typing a word, on this computer (an M-series Mac,
  release build, 18 threads at once), to compare models; not a phone timing.

## Results

### Baseline: the shipped AOSP dictionary (2026-11-16)

`main_en-US.dict`, 2,956,881 bytes: Google's v202 dictionary, at most three next words a word.

| Metric | tatoeba | globalvoices | all |
|---|---:|---:|---:|
| Words scored | 37781 | 86766 | 124547 |
| Next word top-1 | 5.1% | 4.6% | 4.8% |
| Next word top-3 | 13.9% | 11.9% | 12.5% |
| Keystroke savings | 19.8% | 22.4% | 21.7% |
| Typos made (neutral) | 18.0% | 21.1% | 20.1% |
| Word accuracy after autocorrect (neutral) | 96.9% | 96.4% | 96.6% |
| Harmful corrections (neutral, of clean words) | 0.2% | 0.6% | 0.5% |
| Missed typos (neutral, of typos) | 13.8% | 12.5% | 12.8% |
| Wrong fixes (neutral, of typos) | 2.5% | 2.2% | 2.3% |
| Typos made (left thumb) | 34.8% | 39.2% | 37.9% |
| Word accuracy after autocorrect (left thumb) | 92.4% | 92.5% | 92.5% |
| Harmful corrections (left thumb, of clean words) | 0.2% | 0.6% | 0.5% |
| Missed typos (left thumb, of typos) | 18.5% | 16.2% | 16.8% |
| Wrong fixes (left thumb, of typos) | 2.8% | 2.1% | 2.3% |
| Swipe words | 35956 | 84205 | 120161 |
| Swipe top-1 | 89.7% | 89.2% | 89.4% |
| Swipe top-3 | 96.9% | 94.1% | 94.9% |
| Strip time per keystroke, mean (ms) | 0.86 | 0.96 | 0.93 |
| Strip time per keystroke, p95 (ms) | 1.22 | 1.37 | 1.34 |

What stands out: next-word prediction is weak (one word in eight is in the strip after a space),
because the dictionary holds at most three next words for a word, and none for most. Correction
and swipe are already good, so there the question is whether better statistics keep them there
or help a little.

### A correction fix, on the AOSP dictionary (2026-11-16)

Building Stage B turned up a bug in when a word typed right may still be auto-corrected
(`is_much_rarer`, in `crates/suggest/src/suggester.rs`, and the Kotlin reference it matches). The
rule asks whether the word typed has a bigram and how probable it is, but asked it of the decoder's
input, which lower-cases a leading capital. For a name the dictionary only knows capitalised,
"South Korea" asked about "korea": not a word, so "Korea" looked infinitely rarer than "Korean"
and was corrected to it. It now asks about the word as typed when the lower-cased form isn't a
word. On the AOSP dictionary this changes only the left-thumb harmful corrections (0.5% to 0.4%)
and the timings; the full table is the AOSP column below.

### Stage B: the AOSP dictionary rebuilt from the corpus (2026-11-16)

`main_en-US.dict`, 2,370,665 bytes (586,216 fewer): the same v202 format, the same 160,715 words
with the same flags and shortcuts, built by `lexicon dictionary` (see
[rust.md](rust.md#the-typing-scoreboard)) from the 2,852,509 training sentences (32.2 million
words). Possibly-offensive and not-a-word entries keep their old values and are never a next
word. The debug APK went from 62,389,835 to 61,804,235 bytes (585,600 fewer; the asset is
stored uncompressed).

- **Unigrams.** Each word's value is from how often the corpus writes it, on the old scale: the
  dictionary hands out exactly the old set of values again, the most frequent word getting the
  highest, so the decoder's weights and thresholds, tuned on AOSP's values, still fit. Words the
  corpus never writes (82,307 of them) keep their old order, below the rest. A capital at a
  sentence's start is shared between "Will" and "will" as the rest of the corpus writes them.
- **Next words.** Up to 32 a word, each seen at least twice after it: 335,018 on 41,095 words
  (AOSP: at most 3, 481,875 in all, spread over far more words). Each value is the word's
  probability after its previous word, put on the same scale by a fit of value against log
  frequency, then into AOSP's 4-bit encoding. A pair whose value the encoding can't put above the
  word's own is left out.
- **Pairs as typed.** A pair is counted exactly when the keyboard would ask for it: the previous
  word is what `PreviousWords::from` finds before the word, looked up as the keyboard looks it up
  (as written, then lower-cased).

| Metric | tatoeba | globalvoices | all |
|---|---:|---:|---:|
| Words scored | 37781 | 86766 | 124547 |
| Next word top-1 | 14.1% | 12.9% | 13.3% |
| Next word top-3 | 23.9% | 19.8% | 21.0% |
| Keystroke savings | 29.2% | 28.8% | 28.9% |
| Typos made (neutral) | 18.0% | 21.1% | 20.1% |
| Word accuracy after autocorrect (neutral) | 97.4% | 96.6% | 96.9% |
| Harmful corrections (neutral, of clean words) | 0.3% | 0.7% | 0.6% |
| Missed typos (neutral, of typos) | 11.6% | 11.3% | 11.3% |
| Wrong fixes (neutral, of typos) | 1.8% | 2.1% | 2.0% |
| Typos made (left thumb) | 34.8% | 39.2% | 37.9% |
| Word accuracy after autocorrect (left thumb) | 93.9% | 93.4% | 93.5% |
| Harmful corrections (left thumb, of clean words) | 0.3% | 0.6% | 0.5% |
| Missed typos (left thumb, of typos) | 15.1% | 14.1% | 14.4% |
| Wrong fixes (left thumb, of typos) | 2.0% | 1.8% | 1.9% |
| Swipe words | 35956 | 84205 | 120161 |
| Swipe top-1 | 93.5% | 90.2% | 91.2% |
| Swipe top-3 | 97.5% | 94.5% | 95.4% |
| Strip time per keystroke, mean (ms) | 1.00 | 1.03 | 1.02 |
| Strip time per keystroke, p95 (ms) | 1.38 | 1.50 | 1.47 |

Against the AOSP dictionary, all held-out text:

| Metric | AOSP (Stage A) | AOSP, correction fix | Stage B |
|---|---:|---:|---:|
| Next word top-1 | 4.8% | 4.8% | **13.3%** |
| Next word top-3 | 12.5% | 12.5% | **21.0%** |
| Keystroke savings | 21.7% | 21.7% | **28.9%** |
| Word accuracy (neutral) | 96.6% | 96.6% | **96.9%** |
| Harmful corrections (neutral) | 0.5% | 0.5% | 0.6% |
| Missed typos (neutral) | 12.8% | 12.8% | **11.3%** |
| Wrong fixes (neutral) | 2.3% | 2.3% | **2.0%** |
| Word accuracy (left thumb) | 92.5% | 92.5% | **93.5%** |
| Harmful corrections (left thumb) | 0.5% | 0.4% | 0.5% |
| Missed typos (left thumb) | 16.8% | 16.8% | **14.4%** |
| Wrong fixes (left thumb) | 2.3% | 2.3% | **1.9%** |
| Swipe top-1 | 89.4% | 89.4% | **91.2%** |
| Swipe top-3 | 94.9% | 94.9% | **95.4%** |
| Strip time, mean (ms) | 0.93 | 0.90 | 1.02 |
| Strip time, p95 (ms) | 1.34 | 1.29 | 1.47 |

**What got worse, and why.**

- *Harmful corrections* rise 0.1 point on both hands. Most are words the dictionary doesn't
  have, mostly names ("Layla", "Rima") and new words ("hashtag"), corrected into a word the
  context now makes likely: with many more next words, the keyboard is more often sure enough of
  a suggestion to replace an unknown word, the same confidence that fixes more typos. (Before the
  correction fix above, dictionary words typed right were corrected away too; in one run's dump
  those fell from 331 to 76 with it.) The net is more right words: accuracy rises 0.3 point
  neutral and 1.0 point left-thumb.
- *Strip time* rises about 0.1 ms per keystroke: more next words to walk. It stays far inside the
  budget.

**Choosing the settings.** Every variant built from the same counts and scored on the same text
("all" column):

| Variant | Size (bytes) | Next top-1 | Next top-3 | Savings | Accuracy neutral / left | Harmful neutral / left | Swipe top-1 |
|---|---:|---:|---:|---:|---:|---:|---:|
| No next words | 1,043,435 | 0.0% | 0.0% | 20.2% | 96.7% / 93.0% | 0.5% / 0.5% | 91.0% |
| Up to 16 | 2,019,303 | 13.2% | 21.0% | 28.3% | 96.8% / 93.4% | 0.6% / 0.5% | 91.2% |
| **Up to 32 (shipped)** | 2,370,665 | 13.3% | 21.0% | 28.9% | 96.9% / 93.5% | 0.6% / 0.5% | 91.2% |
| Up to 64 | 2,760,780 | 13.3% | 21.0% | 29.6% | 96.8% / 93.6% | 0.7% / 0.6% | 91.0% |
| Up to 32, pairs seen once | 4,048,654 | 13.3% | 21.1% | 29.0% | 96.9% / 93.5% | 0.6% / 0.5% | 91.2% |
| Up to 32, pairs seen 3 times | 1,956,532 | 13.2% | 21.0% | 28.9% | 96.9% / 93.5% | 0.6% / 0.5% | 91.1% |

Unigrams alone already lift correction and swipe; with no next words the strip shows nothing
after a space. 32 next words save 0.6 point more keystrokes than 16 and lose nothing; 64 save
0.7 more again but correct more clean words and swipe a little worse. Counting pairs seen only
once costs 1.7 MB for 0.1 point; requiring three sightings saves 0.4 MB and loses 0.1 point of
next-word top-1 and swipe, so pairs seen twice it is. Also tried and dropped: keeping AOSP's own
next words alongside (twice the size, no gain), and mixing AOSP's word order into the corpus's as
a prior for rarely seen words (next-word accuracy fell, nothing gained).

**Text unlike the corpus.** To see how this does on text unlike what it was built from, a
dictionary built from Tatoeba alone was scored on the GlobalVoices held-out text. Next-word top-1
still doubles (4.6% to 9.3%) and correction holds (96.5% / 92.5% to 96.5% / 92.7%), but swipe
top-1 drops a point (89.3% to 88.3%): next words learnt from small talk mislead the swipe ranker
on news-like text. Built from both sources, as shipped, swipe on GlobalVoices rises instead
(89.2% to 90.2%). So the gains depend on the corpus being near what people type; on very
different text, expect swipe to lose about a point.

### Stage C: a trigram model (2026-11-16)

`main_en-US.lm`, 12,188,272 bytes, alongside the Stage B dictionary: a word trigram model over
the dictionary's words with interpolated Kneser-Ney smoothing, built by `lexicon model` (see
[rust.md](rust.md#the-n-gram-model)) from the same training sentences. 77,757 words (the
dictionary's that the corpus writes, less the possibly-offensive and not-a-word ones, so it can
never predict them), 712,166 pairs and 1,081,773 triples, each kept if seen at least three times;
each log probability and backoff is a byte. Discounts 0.716 for pairs and 0.790 for triples.

- **Next words** after a space are the model's, from the last two words (a sentence start counts
  as one), likeliest first. The user's learned next words still come before them.
- **Typing.** The decoder scores each candidate partly by the dictionary's value for it after
  the previous word. For each candidate the session redoes that term: the word's own value,
  raised by three quarters of the model's lift (how much likelier the model finds it after these
  words than anywhere, on the dictionary's scale), and the candidates are sorted again.
- **Swipe.** The ranker's n-gram for a guess is the word's own value raised by the model's lift,
  when the lift is positive (the context favours it); otherwise it has none.

| Metric | tatoeba | globalvoices | all |
|---|---:|---:|---:|
| Words scored | 37781 | 86766 | 124547 |
| Next word top-1 | 22.8% | 17.4% | 19.0% |
| Next word top-3 | 38.4% | 27.3% | 30.7% |
| Keystroke savings | 39.7% | 35.0% | 36.2% |
| Typos made (neutral) | 18.0% | 21.1% | 20.1% |
| Word accuracy after autocorrect (neutral) | 97.5% | 96.7% | 96.9% |
| Harmful corrections (neutral, of clean words) | 0.3% | 0.7% | 0.6% |
| Missed typos (neutral, of typos) | 11.4% | 11.3% | 11.3% |
| Wrong fixes (neutral, of typos) | 1.5% | 1.9% | 1.8% |
| Typos made (left thumb) | 34.8% | 39.2% | 37.9% |
| Word accuracy after autocorrect (left thumb) | 94.2% | 93.5% | 93.7% |
| Harmful corrections (left thumb, of clean words) | 0.2% | 0.6% | 0.5% |
| Missed typos (left thumb, of typos) | 14.6% | 14.1% | 14.2% |
| Wrong fixes (left thumb, of typos) | 1.6% | 1.6% | 1.6% |
| Swipe words | 35956 | 84205 | 120161 |
| Swipe top-1 | 95.0% | 90.6% | 91.9% |
| Swipe top-3 | 97.6% | 94.5% | 95.4% |
| Strip time per keystroke, mean (ms) | 1.21 | 1.29 | 1.27 |
| Strip time per keystroke, p95 (ms) | 1.78 | 1.86 | 1.84 |
| Next-word strip time, mean (ms) | 0.03 | 0.03 | 0.03 |
| Next-word strip time, p95 (ms) | 0.08 | 0.11 | 0.10 |

Against the earlier stages, all held-out text:

| Metric | AOSP (Stage A) | Stage B | Stage C |
|---|---:|---:|---:|
| Next word top-1 | 4.8% | 13.3% | **19.0%** |
| Next word top-3 | 12.5% | 21.0% | **30.7%** |
| Keystroke savings | 21.7% | 28.9% | **36.2%** |
| Word accuracy (neutral) | 96.6% | 96.9% | 96.9% |
| Harmful corrections (neutral) | 0.5% | 0.6% | 0.6% |
| Missed typos (neutral) | 12.8% | 11.3% | 11.3% |
| Wrong fixes (neutral) | 2.3% | 2.0% | **1.8%** |
| Word accuracy (left thumb) | 92.5% | 93.5% | **93.7%** |
| Harmful corrections (left thumb) | 0.5% | 0.5% | 0.5% |
| Missed typos (left thumb) | 16.8% | 14.4% | **14.2%** |
| Wrong fixes (left thumb) | 2.3% | 1.9% | **1.6%** |
| Swipe top-1 | 89.4% | 91.2% | **91.9%** |
| Swipe top-3 | 94.9% | 95.4% | 95.4% |
| Strip time, mean (ms) | 0.93 | 1.02 | 1.27 |
| Strip time, p95 (ms) | 1.34 | 1.47 | 1.84 |

Nothing got worse. **Latency.** With 18 threads sharing the machine, the strip takes about a
quarter of a millisecond longer a keystroke (1.04 to 1.27 ms mean in the same script run), mostly
the threads' cache misses in the larger tables. Timed alone
(`--threads 1`, 500 sentences a source, back to back), Stage B takes 0.74 ms mean and 1.03 ms p95
a keystroke and Stage C 0.79 ms and 1.08 ms: 0.05 ms more, against a budget of 5 ms. The strip
after a space takes 0.02 ms either way. **Memory.** The model is read once into native memory,
12.2 MB, at its exact size and without a copy; nothing is on the JVM heap. **APK.** The asset is
stored uncompressed: the debug APK went from 61,804,235 to 74,009,087 bytes (12,204,852 more:
the model and 16,580 bytes of code).

**What the model changes.** "See you" now predicts "again", "in", "at", "tomorrow"; the
dictionary alone predicted "have", "know", "can", the same as after "Can you" and "What do you",
since it only knows the last word. After "Can you" the model predicts "tell", "give", "help".
Typing "fo" after "I want to" now offers "do" and "go" before "to".

**Choosing the settings.** Every variant scored on the same text ("all" column). Model size
first, each with the typing weight 1 and the swipe threshold 1 the grid started from:

| Model (pairs, triples kept if seen at least) | Size (bytes) | Next top-1 | Next top-3 | Savings | Missed neutral / left | Wrong fixes neutral / left | Swipe top-1 |
|---|---:|---:|---:|---:|---:|---:|---:|
| Pairs only (3) | 5,730,388 | 13.6% | 23.7% | 32.1% | 11.7% / 15.2% | 1.8% / 1.6% | 90.5% |
| 2, 3 | 14,137,240 | 19.0% | 30.7% | 36.9% | 11.4% / 14.2% | 1.8% / 1.6% | 91.6% |
| 2, 4 | 12,152,536 | 18.9% | 30.5% | 36.7% | 11.4% / 14.3% | 1.9% / 1.6% | 91.5% |
| **3, 3 (shipped)** | 12,188,272 | 19.0% | 30.7% | 36.6% | 11.5% / 14.3% | 1.9% / 1.6% | 91.5% |
| 3, 4 | 10,203,568 | 18.9% | 30.4% | 36.5% | 11.5% / 14.4% | 1.9% / 1.6% | 91.5% |
| 4, 3 | 11,154,896 | 19.0% | 30.6% | 36.5% | 11.5% / 14.4% | 1.9% / 1.6% | 91.6% |
| 4, 4 | 9,341,924 | 18.9% | 30.4% | 36.3% | 11.5% / 14.5% | 1.9% / 1.6% | 91.5% |

(The pairs-only row is with the shipped tuning.) Accuracy and harmful corrections were the same
in every row (neutral 96.9% / 0.6%, left thumb 93.7% / 0.5%) but two: left-thumb accuracy is
93.6% for 4, 4 and 93.3% for pairs only. The word before last is what pays: without triples the model predicts barely better than
the dictionary and swipes worse. Pairs seen twice add 0.3 point of savings and at most 0.1
elsewhere for 1.9 MB more APK and memory; triples seen twice need 16.8 MB and more, over the
15 MB budget, so 4-grams were not tried: after triples seen three times there are 2.8 MB left for
them, and pruning harder to make room loses up to 0.3 point.

Then the session's tuning, on the shipped model:

| Typing weight, swipe threshold | Savings | Missed neutral / left | Wrong fixes neutral / left | Accuracy left | Swipe top-1 |
|---|---:|---:|---:|---:|---:|
| 0 (typing untouched), 1 | 32.0% | 11.3% / 14.4% | 2.0% / 1.9% | 93.5% | 91.5% |
| 0.5, 1 | 35.2% | 11.3% / 14.3% | 1.8% / 1.6% | 93.7% | 91.5% |
| **0.75, 0 (shipped)** | 36.2% | 11.3% / 14.2% | 1.8% / 1.6% | 93.7% | 91.9% |
| 1, 1 | 36.6% | 11.5% / 14.3% | 1.9% / 1.6% | 93.7% | 91.5% |
| 1.5, 1 | 36.9% | 11.7% / 14.6% | 2.1% / 1.7% | 93.5% | 91.5% |
| 1, 0 | 36.6% | 11.5% / 14.3% | 1.9% / 1.6% | 93.7% | 91.9% |
| 1, 0.5 | 36.6% | 11.5% / 14.3% | 1.9% / 1.6% | 93.7% | 91.7% |
| 1, 2 | 36.6% | 11.5% / 14.3% | 1.9% / 1.6% | 93.7% | 91.6% |
| 1, never | 36.6% | 11.5% / 14.3% | 1.9% / 1.6% | 93.7% | 91.0% |

Next words don't depend on either setting (19.0% / 30.7% throughout). The full weight saves
0.4 point more keystrokes than three quarters but leaves 0.2 point more typos uncorrected than
Stage B did; three quarters leaves every correction measure at least as good as Stage B. For swipe, any positive lift counts best; ignoring the model (never) is worse than the
dictionary's own pairs (91.2%).
