# Swipe accuracy

Cheap changes to how the keyboard decodes a swipe, each measured by the typing scoreboard
(`crates/scoreboard`; [how to run it](rust.md#the-typing-scoreboard)) on the held-out text
[language-model-results.md](language-model-results.md) describes. A change was kept only if it
raised top-1 on **both** swipe simulators and lowered top-3 on neither, with a neutral hand and a
left thumb. Four were kept, the personal-words boost among them only on top of the new frequency
curve. Double letters and subset paths were not built: nothing measurable separates them.

FlorisBoard's classifier stays in the code, bit for bit: `glide::Tuning::ORIGINAL`,
`swipe_words::Curve::FLAT` and `swipe_words::frequency`, which the Kotlin reference tests
(`SwipeIndexTest`, `SwipeWordsTest`, `SwipeRulesTest`) still hold to it unchanged. The keyboard
uses `Tuning::DEFAULT`, `Curve::DEFAULT` and `Boost::DEFAULT` through `SwipeIndex.tunedSuggestions`
and `SwipeWords.tunedFrequency`.

## Two simulators

A change tuned against one simulated finger can learn that finger's habits rather than a
person's. So the scoreboard swipes every word with two that share nothing but the keyboard and
the word: one drawn by rule, one by a published model of human movement.

**`trace`, by rule** (`crates/scoreboard/src/swipe/trace.rs`). The swipe the scoreboard already
had: it aims at each key with a Gaussian miss of a fifth of a key, pulls each turn in towards its
neighbours by up to a third of a key, and draws a Catmull-Rom curve through the aims with a little
tremor. It had no times, so it now has a clock: the two-thirds power law of drawing movements
(F. Lacquaniti, C. Terzuolo & P. Viviani, "The law relating the kinematic and figural aspects of
drawing movements", *Acta Psychologica* 54, 115–130, 1983), speed falling as the cube root of
curvature, capped on straight runs. It slows at turns, not at a letter it passes straight through.

**`lognormal`, a model of movement** (`crates/scoreboard/src/swipe/lognormal.rs`). The kinematic
theory of rapid human movements in its sigma-lognormal form: R. Plamondon, "A kinematic theory of
rapid human movements. Part I", *Biological Cybernetics* 72, 295–307, 1995; R. Plamondon &
M. Djioua, "A multi-level representation paradigm for handwriting stroke generation", *Human
Movement Science* 25, 586–607, 2006; used as L. A. Leiva, D. Martín-Albo & R. Plamondon use it to
synthesise human-like gestures ("Gestures à Go Go", *ACM Transactions on Intelligent Systems and
Technology* 7(2), 2015, doi:10.1145/2799648). The finger's velocity is a sum of overlapping
strokes, one to each key, each with a lognormal speed profile and a slight bow; Fitts' law sets
each stroke's time. It slows at every letter, turns before it gets there, and cuts corners by the
overlap, not by a rule. S. Quinn & S. Zhai, "Modeling gesture-typing movements" (*Human–Computer
Interaction* 33(3), 234–280, 2018) is the background for how gesture typing moves.

**Left thumb**, for both: aims fall short of far keys and scatter more there, as the scoreboard's
taps do, and the lognormal strokes bow as a thumb turning about the keyboard's bottom left would.

How differently they swipe, with a neutral hand on the report text:

| | `trace` | `lognormal` |
|---|---|---|
| Mean miss of the nearest point to each key's centre | 0.226 key widths | 0.366 key widths |
| Speed at a letter, as a share of the peak between letters | 0.32 | 0.23 |
| Time per key-to-key segment | 609 ms | 316 ms |

`trace`'s clock is slower than people swipe; dwell weighs a swipe's speed against its own mean,
so only the profile matters. The lognormal parameters are within the ranges those papers fit to
fast handwriting and gestures.
The first setting was harsher than people are (top-1 81.5%, misses of 0.47 key widths), so the
aim spread went from 33 px to 28 px (1.2 mm) and each stroke's start from 0.7–1.0 to 0.8–1.1 of
the last one's median time. Then both simulators were frozen, before any change was measured.

## How the changes were measured

- **Tuning text**: sentences 5,000–6,999 of each source's held-out set
  (`--skip 5000 --limit 2000`): 47,411 swipes for each variant. Every strength was chosen here.
- **Report text**: the first 5,000 of each, as in language-model-results.md: 120,161 swipes for each
  of the four simulator-and-hand variants. Each change was run on it once, at its chosen strength.
- **History**: the boost needs a user who has typed. `--history 7000` has them type the 5,000
  sentences from 7,000 on of each source first, learned as the keyboard's word learner spells them
  (a capital that only starts a sentence is dropped). It is clear of both texts above. On its own
  it changes nothing: FlorisBoard's classifier swipes the report text identically with and without.
- Rows are top-1 / top-3, in percent; a bold row is the strength kept. Each variant has its own
  seeded stream, so rows compare word for word.

## The changes

### 1. The frequency curve: one slope all the way up (kept)

FlorisBoard counts every word of probability 170 or more the same (frequency 1) and charges
0.007 nats a step below that. Words that share a path ("the" and "three", "to" and "too", "of"
and "off") then tie on frequency and are left to the ranker. `Curve::DEFAULT` keeps charging
0.007 a step up to 255, so a commoner word always counts for a little more.

Tuning text, the cost a step above 170 (0 is FlorisBoard's):

| Above | `trace` neutral | `trace` left | `lognormal` neutral | `lognormal` left |
|---|---|---|---|---|
| 0 | 91.80 / 95.34 | 90.87 / 94.85 | 87.22 / 92.49 | 85.56 / 91.60 |
| 0.002 | 92.03 / 95.34 | 91.07 / 94.86 | 87.46 / 92.50 | 85.98 / 91.61 |
| 0.004 | 92.13 / 95.34 | 91.20 / 94.86 | 87.61 / 92.51 | 86.22 / 91.62 |
| **0.007** | **92.20 / 95.34** | **91.28 / 94.86** | **87.72 / 92.52** | **86.41 / 91.64** |
| 0.01 | 92.18 / 95.34 | 91.25 / 94.86 | 87.71 / 92.52 | 86.44 / 91.65 |
| 0.015 | 92.00 / 95.34 | 91.03 / 94.86 | 87.55 / 92.54 | 86.27 / 91.66 |
| 0.03 | 91.26 / 95.33 | 90.27 / 94.85 | 86.76 / 92.55 | 85.54 / 91.67 |

0.007, the same slope as below 170, is the best or within 0.03 of it everywhere.

### 2. Ends: the swipe starts and stops on the word's end keys (kept)

Gesture decoders weigh where a swipe starts and ends (SHARK²: P.-O. Kristensson & S. Zhai,
"SHARK²: a large vocabulary shorthand writing system for pen-based computers", UIST 2004).
FlorisBoard's classifier only files words by their end keys. `Tuning::ends` makes a word less
likely by a Gaussian of the distances from the swipe's first and last points to its first and
last keys, in half-key spreads.

| Ends | `trace` neutral | `trace` left | `lognormal` neutral | `lognormal` left |
|---|---|---|---|---|
| 0 | 91.80 / 95.34 | 90.87 / 94.85 | 87.22 / 92.49 | 85.56 / 91.60 |
| 0.1 | 92.54 / 95.51 | 91.66 / 95.03 | 88.32 / 92.97 | 86.64 / 92.08 |
| 0.2 | 92.77 / 95.60 | 91.90 / 95.11 | 88.82 / 93.24 | 87.05 / 92.33 |
| **0.4** | **92.95 / 95.66** | **92.08 / 95.15** | **89.18 / 93.51** | **87.30 / 92.52** |
| 0.8 | 92.97 / 95.68 | 91.93 / 95.06 | 89.24 / 93.62 | 87.08 / 92.54 |
| 1.6 | 92.86 / 95.62 | 91.35 / 94.76 | 89.07 / 93.54 | 86.36 / 92.06 |

0.8 is a hair better with a neutral hand and worse with a left thumb, which misses far keys more.

### 3. Dwell: the finger slows at the letters it means (kept)

The classifier compares the swipe and each word's path point for point, every point alike.
`Tuning::dwell` counts the swipe's slow stretches for more: each resampled point weighs its
slowness (time per distance, over half a key either side) over the swipe's mean, raised to
`dwell`, within ¼–4. It needs each point's time, which the keyboard didn't keep: the renderer now
sends one before each glide point (`GLIDE_TIME`, opted into by the gesture config), and the
classifier's gesture keeps it.

| Dwell | `trace` neutral | `trace` left | `lognormal` neutral | `lognormal` left |
|---|---|---|---|---|
| 0 | 91.80 / 95.34 | 90.87 / 94.85 | 87.22 / 92.49 | 85.56 / 91.60 |
| 0.5 | 91.81 / 95.35 | 90.92 / 94.86 | 87.88 / 92.89 | 86.36 / 92.08 |
| 1 | 91.86 / 95.36 | 90.97 / 94.87 | 88.31 / 93.14 | 86.95 / 92.44 |
| **1.5** | **91.91 / 95.38** | **91.00 / 94.89** | **88.48 / 93.26** | **87.18 / 92.58** |
| 2 | 91.96 / 95.40 | 91.03 / 94.90 | 88.57 / 93.31 | 87.26 / 92.64 |

It helps `lognormal` most, which slows at every letter, and `trace` a little, which slows only at
turns. 2 was slightly better still (at most 0.09 point); 1.5 was chosen by judgement, not by the
table, since the gains flatten and a stronger weight trusts the simulators' speed profiles more
than a real finger may deserve.

### 4. Reach: the swipe goes where the word goes, and only there (kept)

The classifier's distances average over the whole path, so a word can miss a key the swipe
clearly turned at, or need a key the swipe never neared, and still win. `Tuning::reach` makes a
word less likely by the farthest the swipe strays from the word's path and the farthest any of the
word's keys lies off the swipe, each beyond half a key, squared. It is the costliest addition
(point-to-segment distances on the swipe as drawn), so it is only computed for a word that would
make the ranking.

| Reach | `trace` neutral | `trace` left | `lognormal` neutral | `lognormal` left |
|---|---|---|---|---|
| 0 | 91.80 / 95.34 | 90.87 / 94.85 | 87.22 / 92.49 | 85.56 / 91.60 |
| 0.25 | 92.14 / 95.43 | 91.22 / 94.96 | 87.60 / 92.68 | 85.83 / 91.84 |
| **0.5** | **92.34 / 95.51** | **91.47 / 95.03** | **87.74 / 92.76** | **85.92 / 91.88** |
| 1 | 92.60 / 95.60 | 91.73 / 95.12 | 87.77 / 92.71 | 85.81 / 91.74 |
| 4 | 93.16 / 95.75 | 92.10 / 95.23 | 86.31 / 91.63 | 83.51 / 89.98 |

This is what the second simulator is for: strong reach keeps paying on `trace` and costs
`lognormal` 0.9 to 2.1 points, whose finger misses keys by more (0.37 key widths against 0.23).
With the other three changes in, on the tuning text:

| Reach, with 1–3 | `trace` neutral | `trace` left | `lognormal` neutral | `lognormal` left |
|---|---|---|---|---|
| 0.25 | 93.68 / 95.73 | 92.83 / 95.27 | 90.64 / 94.11 | 89.31 / 93.37 |
| **0.5** | **93.78 / 95.78** | **92.93 / 95.31** | **90.72 / 94.11** | **89.34 / 93.35** |
| 1 | 93.92 / 95.81 | 93.04 / 95.38 | 90.60 / 94.04 | 89.08 / 93.23 |

0.5 is best on `lognormal`; 1 gains 0.1 on `trace` and loses 0.1–0.3 on `lognormal`.

### 5. Personal words: the user's own words count more the more they use them (kept, with 1)

A learned word counts as probability 120 once used twice, however often it is used after. With a
boost, a word the user spells that way gains `per_doubling` steps each time its uses double from 2,
up to `most`: 5 steps at 2 uses, 10 at 4, 15 at 8 and the most, 20, from 16. It lives with the
learned words, so clearing them (or turning learning off) clears it, and it changes only when the
uses double, so it seldom makes the classifier re-index.

On FlorisBoard's flat curve it loses: a boosted "three" or "too" reaches the flat top and ties
"the" and "to" (report text, history on both):

| | `trace` neutral | `trace` left | `lognormal` neutral | `lognormal` left |
|---|---|---|---|---|
| FlorisBoard | 91.88 / 95.44 | 91.10 / 95.03 | 87.50 / 92.61 | 85.59 / 91.59 |
| Boost 5, 20 | 91.02 / 95.46 | 90.27 / 95.06 | 86.75 / 92.74 | 84.76 / 91.75 |

On the one-slope curve, a commoner word stays ahead and the boost pays. Tuning text, with 1–4:

| Boost | `trace` neutral | `trace` left | `lognormal` neutral | `lognormal` left |
|---|---|---|---|---|
| none | 93.78 / 95.78 | 92.92 / 95.31 | 90.72 / 94.10 | 89.34 / 93.35 |
| **5, 20** | **93.81 / 95.79** | **92.97 / 95.35** | **90.88 / 94.22** | **89.60 / 93.48** |
| 10, 40 | 93.77 / 95.80 | 92.93 / 95.37 | 90.96 / 94.28 | 89.70 / 93.57 |
| 20, 60 | 93.56 / 95.79 | 92.76 / 95.39 | 90.76 / 94.30 | 89.54 / 93.64 |

Only 5, 20 passes on both simulators. On the curve alone (history on both) it passes too: top-1
rises 0.02–0.42 and top-3 0.03–0.27 on either text (report text below). Its gain on `trace` is
small (0.02–0.10 point), and the history is in the test text's own domain and long (10,000
sentences), so a real user's gain depends on how much they type and how unlike the dictionary
it is.

### Not built: double letters and subset paths

A doubled letter ("hello", "book") leaves no mark in a swipe: neither simulator draws one, and a
real finger doesn't loop or pause reliably either. A word whose path is part of another's ("to"
and "too", "in" and "inn") is the same shape. Nothing geometric can separate either, so there was
nothing to measure; frequency (the curve), reach (which charges the longer word when the swipe
doesn't go out of its way) and the ranker's words-before are what separate them.

## Results on the report text

Each change alone against FlorisBoard's classifier, and all kept together:

| | `trace` neutral | `trace` left | `lognormal` neutral | `lognormal` left |
|---|---|---|---|---|
| FlorisBoard | 91.88 / 95.44 | 91.10 / 95.03 | 87.50 / 92.61 | 85.59 / 91.59 |
| Curve, one slope | 92.23 / 95.45 | 91.43 / 95.03 | 87.91 / 92.64 | 86.32 / 91.65 |
| Ends 0.4 | 93.07 / 95.75 | 92.23 / 95.32 | 89.39 / 93.52 | 87.36 / 92.60 |
| Dwell 1.5 | 92.02 / 95.50 | 91.23 / 95.08 | 88.73 / 93.41 | 87.22 / 92.61 |
| Reach 0.5 | 92.44 / 95.62 | 91.72 / 95.20 | 87.94 / 92.86 | 85.90 / 91.85 |
| Curve (history) | 92.24 / 95.45 | 91.43 / 95.03 | 87.92 / 92.64 | 86.32 / 91.65 |
| Curve + boost 5, 20 (history) | 92.26 / 95.48 | 91.51 / 95.07 | 88.21 / 92.84 | 86.74 / 91.91 |
| All but the boost | 93.85 / 95.86 | 93.07 / 95.47 | 90.88 / 94.20 | 89.34 / 93.45 |
| All but the boost (history) | 93.86 / 95.86 | 93.07 / 95.47 | 90.88 / 94.20 | 89.35 / 93.45 |
| **All (history): the keyboard** | **93.90 / 95.88** | **93.15 / 95.49** | **91.08 / 94.30** | **89.63 / 93.61** |

All together, top-1 rises 2.0 to 2.1 points on `trace` and 3.6 to 4.0 on `lognormal`; top-3, 0.4
to 0.5 and 1.7 to 2.0. About a quarter (23–29%) of the swipes FlorisBoard's classifier got wrong
are now right.

Each kept change still pays with the others in (tuning text, all but the boost; leaving one out):

| | `trace` neutral | `trace` left | `lognormal` neutral | `lognormal` left |
|---|---|---|---|---|
| All four | 93.78 / 95.78 | 92.93 / 95.31 | 90.72 / 94.11 | 89.34 / 93.35 |
| Without the curve | 92.48 / 95.78 | 91.63 / 95.32 | 89.36 / 94.17 | 87.63 / 93.40 |
| Without ends | 92.87 / 95.58 | 92.03 / 95.12 | 89.67 / 93.63 | 88.53 / 92.96 |
| Without dwell | 93.72 / 95.76 | 92.89 / 95.32 | 90.14 / 93.77 | 88.64 / 92.92 |
| Without reach | 93.52 / 95.71 | 92.66 / 95.25 | 90.58 / 94.06 | 89.33 / 93.36 |

With the others in, the curve trades a little top-3 for top-1: without it, `lognormal`'s top-3
is 0.05–0.06 higher and its top-1 1.4–1.7 lower. Alone it lowered top-3 nowhere (above), which
is what the rule asks; top-1 is the word committed, so the trade is kept.

## Speed

Swipe decode per gesture on this computer (an M-series Mac), one thread, the first 1,000 report
sentences of each source with history, all four variants: point filter, scoring and re-ranking by
the words before, as the keyboard does after each swipe.

The computer was shared with other work, so each decoder ran six times, alternating (ABBA), on
the same 95,176 swipes; the fastest run is the least disturbed:

| | Mean | p95 | Means over the six runs |
|---|---|---|---|
| FlorisBoard's classifier | 0.81 ms | 3.75 ms | 0.81–1.44 ms |
| The keyboard's decoder | 0.88 ms | 3.91 ms | 0.88–0.90 ms |

Process CPU time, which load inflates less, agrees: the fastest of four runs each took 77.6 s and
88.5 s, 0.11 ms more a swipe. So the additions cost about 0.1 ms a swipe, a tenth more. This is
the host, not a phone; on a phone it has yet to be timed.

## Reproducing

```bash
cd lib/rust
RUN=../../build/language-model/run-<time>/corpus   # scripts/build-language-model.sh makes it
SB="cargo run --release -p scoreboard -- --model ../../app/src/main/assets/dicts/main_en-US.lm \
    --test tatoeba=$RUN/test-tatoeba.txt --test globalvoices=$RUN/test-globalvoices.txt --swipe-only"
$SB --limit 5000 --original --history 7000         # FlorisBoard's classifier
$SB --limit 5000 --history 7000                    # the keyboard
$SB --skip 5000 --limit 2000 --original --ends 0.8 # one change, on the tuning text
```

Mutations: `python3 -I scripts/glide-mutations.py OUTPUT` breaks each kept change in turn and
requires a test to fail by assertion, then restores the bytes and requires it to pass again.
