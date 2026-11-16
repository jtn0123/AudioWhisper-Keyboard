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
  ("I", "a") aren't swiped.
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
