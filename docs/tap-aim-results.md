# Learning where the user taps: results

The keyboard learns, per key and per orientation, how far this user's taps land from the key
they meant, and allows for it in two places:

* **(a) the decoder input**: the tap positions the suggestion decoder hears are moved back by the
  learned offset (`TapAim::aimed`).
* **(b) the hit test**: a tap near a key's edge types the neighbour instead, but only when both
  keys' learned aims agree (`keys::aim::retarget`).

Both ship, default on, behind *Typing → Learn how I tap*. Why, and what would change it:
[decisions.md](decisions.md#learning-where-the-user-taps-2026-11-16). How it is built:
[rust.md](rust.md#learning-where-the-user-taps).

## What it learns from, and its limits

It learns only from a word the user committed **as typed** and then kept:

* the word has no backspace inside it;
* every character the word ends with was tapped (a long-press popup character has no position);
* the word is confirmed only once typing goes on (two more characters, the next word kept, a
  prediction tapped or a swipe);
* a backspace, an undo, or a suggestion that replaces the word before then throws it away;
* auto-corrected words teach nothing (see *Corrected words* below);
* nothing is learned in incognito, in password fields, with auto-correct off, or with the setting
  off (the NLP session's "aim" operation).

The limits (`crates/suggest/src/tap_aim.rs`):

| Limit | Value | What it does |
|---|---:|---|
| `MIN_SAMPLES` | 16 | A key is not aimed until it has this much weight |
| `WINDOW` | 200 | A key's running mean and spread forget older taps on this scale |
| `FADE` | 20,000 | Every key fades slowly as other keys learn, so an aim not refreshed goes away |
| `CAP` | 0.25 | The most a tap is moved, in key widths or heights |
| `CONFIDENCE` | 1.0 | The aim applied is the mean less this many standard errors |
| `MIN_BIAS` | 0.03 | A key whose aim is closer to centre than this is left alone |
| `SAMPLE_LIMIT` | 0.75 | A sample counts as at most this far from its key's centre |
| `MAX_KEYS` | 64 | Keys learned per orientation |

Portrait and landscape are learned apart. The file is `files/tap-aim.txt`: a header, then one line
of numbers per key (orientation, key code, weight, mean across and down, and their variances). *Forget learned words* erases it, and backups
leave it out.

## Method

`cargo run --release -p scoreboard -- ... --learning` (`crates/scoreboard/src/learning.rs`). Each
simulated user types one stream of 10,000 held-out sentences (Tatoeba and Global Voices, the
scoreboard's own test sets) with seed 20261116. The keyboard learns from the words they keep, as
on the phone (`suggest::tap_learning`). The first 2,000 sentences are a warm-up and are not
scored. The user commits what auto-correction gives, fixes a wrong word with a backspace 70% of
the time (`FIXES`), and leaves the rest. A typo left as typed is then "kept" and teaches the aim
wrongly, as it would on the phone.

Each user is typed three ways, with the taps drawn the same way word by word for all three:
**off**, **(a)**, and **(a)+(b)**.

The users:

* **neutral**: the scoreboard's neutral typist, with no bias to learn. Whatever changes here is
  harm.
* **left thumb**: the scoreboard's left-thumb typist.
* **left thumb, own aim per key**: the same, plus a seeded aim of its own at each key, up to 0.12
  key widths across and 0.1 down.
* **changes grip halfway**: that user, who switches at sentence 5,000 to a right thumb with
  another per-key aim. It is scored before the change, for the 500 sentences right after it, and
  for the rest.
* **right thumb, other aim per key**: the grip changed to, from the start. This is what the
  change's later half should come to.

## The shipped run

Command:

```bash
cd lib/rust
cargo run --release -p scoreboard -- --model ../../app/src/main/assets/dicts/main_en-US.lm \
    --test tatoeba=RUN/corpus/test-tatoeba.txt --test globalvoices=RUN/corpus/test-globalvoices.txt \
    --limit 5000 --learning
```

Word accuracy after auto-correct:

| User | Words | off | (a) | (a)+(b) |
|---|---:|---:|---:|---:|
| neutral | 99,893 | 96.90% | 96.90% | 96.90% |
| left thumb | 99,893 | 93.67% | 93.88% | **94.84%** |
| left thumb, own aim per key | 99,893 | 93.11% | 93.40% | **94.50%** |
| changes grip: before the change | 37,195 | 92.89% | 93.13% | **94.28%** |
| changes grip: first 500 after | 6,427 | 93.61% | 93.82% | **94.20%** |
| changes grip: rest | 56,271 | 93.40% | 93.57% | **94.02%** |
| right thumb, other aim per key | 99,893 | 93.32% | 93.53% | **94.00%** |

Typos made (a word whose taps did not spell it):

| User | off | (a) | (a)+(b) |
|---|---:|---:|---:|
| neutral | 20.47% | 20.47% | 20.47% |
| left thumb | 37.84% | 37.84% | **32.98%** |
| left thumb, own aim per key | 40.56% | 40.56% | **35.74%** |
| changes grip: before the change | 40.65% | 40.65% | **36.23%** |
| changes grip: first 500 after | 39.13% | 39.13% | **37.58%** |
| changes grip: rest | 38.96% | 38.96% | **36.21%** |
| right thumb, other aim per key | 39.06% | 39.06% | **36.14%** |

(a) leaves the typed letters alone, so it cannot change this row. Only (b) changes which key a tap
types.

Harmful corrections (of clean words) and missed typos (of typos):

| User | Harmful off / (a) / (a)+(b) | Missed off / (a) / (a)+(b) |
|---|---|---|
| neutral | 0.57 / 0.57 / 0.57% | 11.30 / 11.30 / 11.30% |
| left thumb | 0.49 / 0.51 / 0.54% | 14.21 / 13.69 / 12.88% |
| left thumb, own aim per key | 0.44 / 0.48 / 0.51% | 14.83 / 14.07 / 12.94% |
| changes grip: before the change | 0.43 / 0.48 / 0.54% | 15.09 / 14.39 / 13.01% |
| changes grip: first 500 after | 0.20 / 0.20 / 0.22% | 14.16 / 13.76 / 13.37% |
| changes grip: rest | 0.50 / 0.51 / 0.52% | 14.33 / 13.88 / 13.77% |
| right thumb, other aim per key | 0.48 / 0.49 / 0.51% | 14.35 / 13.85 / 13.80% |

The harmful rate rises a little for the biased users because it is a share of *clean* words, and
(b) turns about 4,900 typo'd words into clean ones. Those are the hard, edge-of-key words. In
counts, every biased user commits more words right with each step:

| User | Words committed right, off / (a) / (a)+(b) | Words tapped cleanly, off / (a)+(b) |
|---|---|---|
| neutral | 96,795 / 96,795 / 96,795 | 79,445 / 79,445 |
| left thumb | 93,568 / 93,776 / 94,735 | 62,093 / 66,953 |
| left thumb, own aim per key | 93,007 / 93,302 / 94,402 | 59,374 / 64,188 |
| changes grip: before the change | 34,552 / 34,639 / 35,069 | 22,074 / 23,721 |
| changes grip: first 500 after | 6,016 / 6,030 / 6,054 | 3,912 / 4,012 |
| changes grip: rest | 52,559 / 52,652 / 52,907 | 34,346 / 35,898 |
| right thumb, other aim per key | 93,224 / 93,428 / 93,902 | 60,874 / 63,795 |

**The neutral typist is untouched, word for word, under both (a) and (b).**

After a grip change, the first 500 sentences still gain (+0.59 points with (a)+(b)): the old aim
fades rather than fights. The rest gains +0.62, against +0.68 for a user with that grip from the
start.

## Sensitivity: the dead zone

Without `MIN_BIAS`, the neutral typist's own noise produces small aims that (b) acted on now and
then. That run is `learning-final.md`: neutral committed 96,795 / 96,793 / 96,792 words right and
tapped 79,443 cleanly under (a)+(b), against 79,445. The dead zone fixes that. The table below shows
how much it costs the biased users. Neutral is exact (96,795 and 79,445) in every row except the
first.

| Variant | Left thumb (a)+(b) | Own aim | Before change | First 500 after | Rest |
|---|---:|---:|---:|---:|---:|
| No dead zone (neutral loses 3 words) | 94.86% | 94.50% | 94.30% | 94.17% | 94.09% |
| **`MIN_BIAS` 0.03 (shipped)** | **94.84%** | **94.50%** | **94.28%** | **94.20%** | **94.02%** |
| `MIN_BIAS` 0.04 | 94.76% | 94.51% | 94.24% | 94.13% | 93.98% |
| `MIN_BIAS` 0.05 | 94.67% | 94.46% | 94.17% | 94.04% | 93.91% |
| No dead zone, `CONFIDENCE` 2.0 | 94.60% | 94.32% | 94.07% | 94.09% | 93.88% |

0.03 is the smallest dead zone tried that leaves the neutral typist exact. It gives up 0.02 points
on the left thumb and at most 0.07 on any row. A wider standard-error margin (`CONFIDENCE` 2) also
protects the neutral typist, but it gives up 0.18 to 0.26 points on every row but one.

## Corrected words

Should a kept auto-correction teach too? Two variants were tried, both without the dead zone,
against teaching nothing (`--from-corrected`):

* **unchanged**: the corrected word's letters the correction left alone teach.
* **rebased**: those, plus each changed letter's tap, moved onto the key the correction says was
  meant when that key is next to the one tapped.

| Variant | Left thumb (a) | Left thumb (a)+(b) | Own aim (a)+(b) | First 500 after (a)+(b) | Rest (a)+(b) |
|---|---:|---:|---:|---:|---:|
| Nothing (shipped) | 93.88% | 94.86% | 94.50% | 94.17% | 94.09% |
| Unchanged | 93.89% | 94.86% | 94.51% | 94.29% | 94.13% |
| Rebased | 93.93% | 94.95% | 94.54% | 94.35% | 94.21% |

Neutral is 96.90% in all three. "Rebased" adds 0.05–0.18 points. However, it teaches the aim from
what the *language model* believed was meant, so any wrong correction the user lets stand would
teach the aim to make that mistake more often. The gain is small, so only words committed as typed
teach for now. The decision entry says what evidence would change that.

## The main scoreboard, before and after

The main scoreboard (no learning, `--limit 5000`, seed 20261116) is unchanged by this work, as it
should be: with nothing learned, every path behaves as before.

| Metric | Before (docs) | After |
|---|---:|---:|
| Word accuracy after auto-correct, neutral | 96.9% | 96.9% |
| Word accuracy after auto-correct, left thumb | 93.7% | 93.7% |
| Typos made, neutral | 20.1% | 20.1% |
| Typos made, left thumb | 37.9% | 37.9% |
| Next word top-1 / top-3 | 19.0% / 30.7% | 19.0% / 30.7% |
| Keystroke savings | 36.2% | 36.2% |
| Swipe top-1 | 91.9% | 91.9% |

The raw outputs are under the ignored `build/tap-aim/` (`learning-shipped.md`, the variants, and
`scoreboard-after.md`).
