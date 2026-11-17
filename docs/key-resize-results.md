# Keys resized by the letter odds: results

A tap near the border between two keys types the one whose letter is likelier to come next in
the word being typed, as iOS and Gboard do: after "I think th" an E is far likelier than an R, so
near their border E takes a little of R's room, though both keys look the same. It ships, always
on, in both renderers' hit tests. Why, and what would change it:
[decisions.md](decisions.md#keys-resized-by-the-letter-odds-2026-11-16). How it is built:
[rust.md](rust.md#keys-resized-by-the-letter-odds).

## The rule

* **The odds.** For the words before and the letters typed so far, the n-gram model's probability
  of each of its words that start with them, summed by the letter that comes next
  (`ngram::Model::next_units`, exact for the stored model: the explicit n-grams plus the backoffs
  times the rest of the run's lower-order mass). What was typed is looked up in lower case and
  capitalised, so "th" continues "the" and "Thomas" alike (`suggest::letter_odds`). None once no
  word the model knows starts with what was typed: a word leaving the dictionary turns it off.
* **The prior.** A key's prior is ln((1 − 0.1) × share + 0.1 / 26): a tenth of the odds is spread
  over every letter, so a letter no known word continues with is unlikely, never impossible.
  Keys that aren't letters have none and never move.
* **The border.** Bayes' rule with a Gaussian tap: the border between the key hit and a neighbour
  moves towards the less likely one by `factor × (prior_neighbour − prior_hit)` key widths (0.04
  across, 0.025 between rows; a tap spread of 0.2 key across and 0.16 down), **at most 0.15 key**.
  Only the key hit and the neighbours just across its four sides compete; of two, the one the tap
  is further over the moved border for (`keys::resize::retarget`).
* **Never far inside a key.** A tap more than 0.15 of a key inside its key always types it.
* **With the learned aim** (`keys::resize::key_for`): the aim goes first. Where it moves the tap
  (both keys' aims agree, `keys::aim::retarget`), that is the key and the odds never move it again
  or back. Where it leaves the tap, the odds act on where this user meant it: the tap taken back
  by the key's own learned shift, stopped at the key's border. So the odds never undo the aim, and
  when the aim declines a move they act as for a tap on the border, no further.
* **Only while they hold** (`TapTrail::at_word`). The suggester works the odds out after each
  keystroke, off the hit test's path; a key going down uses them only if the letters typed end in
  the word they were worked out for and no other key is down, or, for a word's first letter, the
  key down before it was the space bar. So: no odds for a key pressed while the previous letter is
  still down (rollover: they would be a letter behind), none for the first key after a backspace
  or shift press, none with the cursor inside a word.
* **Two stale cases left open (2026-11-16, looked at and not fixed).** The trail only tells that
  the letters typed *end* in the odds' word. (1) Typing "t" then another "t" before the suggester
  catches up leaves odds for "t" that the trail still matches, as "tt" ends in "t" (only a word
  ending in its own start can do this: doubled first letters, mostly). (2) Two short words typed
  faster than suggesting ("a b ", then a key) leave the odds from the first space, after "a",
  standing at the second, as both are an empty word after the space bar. Neither the scoreboard
  nor any host test can show them: the scoreboard works the odds out at each tap from the true
  context (`scoreboard/src/typing.rs`), so its numbers can't move either way. No fix is clean in
  Rust alone. The trail never sees spaces as text or cursor moves, so counting letters since the
  last space bar fixes (1) only, and costs the odds for a new word typed after tapping the cursor
  somewhere (the count then includes the letters before the move). Fixing both needs the odds to
  name the content they were worked out for and the hit test to know the content now: a request
  id carried from `NlpManager.suggest` through FlorisBoard's provider interface to the JNI
  suggest call, or the native editor session publishing its content to the letter odds. Both
  are more wiring than two cases bounded at 0.15 key are worth today.

The cost lands on the suggest path; the hit test looks up 26 numbers under a mutex. Kotlin passes
the tap trail handle and the learn-my-aim setting to both hit tests, which it already had; nothing
else is new.

## Scoreboard

Held-out text (`build/language-model/run-20261116-051542/corpus/test-*.txt`, Tatoeba and Global
Voices), the shipped dictionary and model, seed 20261116. Tuned on `--skip 5000 --limit 2000`;
reported on `--limit 5000` (124,547 words). "Rare words" are the 4,888 words outside the
dictionary, which autocorrect can only harm. With `--resize`, every word is typed twice with the
same taps, through the plain hit test and through this one, so the lost and won columns count
words committed right by one and not the other.

```bash
cd lib/rust
cargo run --release -p scoreboard -- --model ../../app/src/main/assets/dicts/main_en-US.lm \
    --test tatoeba=RUN/corpus/test-tatoeba.txt --test globalvoices=RUN/corpus/test-globalvoices.txt \
    --limit 5000 --correction-only --resize 0.04,0.025,0.15,0.1 [--learning]
```

### Alone, learning off (`--limit 5000`)

| | Neutral | Left thumb | Right thumb |
|---|---:|---:|---:|
| Typos made, plain → resized | 20.14% → **8.52%** | 37.89% → **20.26%** | 38.14% → **20.51%** |
| Word accuracy after autocorrect | 96.92% → **97.62%** | 93.72% → **95.18%** | 93.60% → **95.12%** |
| Harmful corrections, of clean words | 0.58% → **0.54%** | 0.48% → **0.42%** | 0.48% → **0.44%** |
| Words committed right | 120,715 → 121,581 | 116,728 → 118,549 | 116,574 → 118,465 |
| Words lost / won | 298 / 1,164 | 477 / 2,297 | 475 / 2,366 |

### Rare words (outside the dictionary, 4,888)

| | Neutral | Left thumb | Right thumb |
|---|---:|---:|---:|
| Word accuracy after autocorrect | 64.42% → **65.65%** | 46.75% → **49.30%** | 48.20% → **50.70%** |
| Typos made | 25.68% → 23.85% | 47.01% → 43.74% | 45.74% → 42.23% |
| Words committed right | 3,149 → 3,209 | 2,285 → 2,410 | 2,356 → 2,478 |
| Words lost / won | 115 / 175 | 100 / 225 | 121 / 243 |
| Tapped cleanly | 3,633 → 3,722 | 2,590 → 2,750 | 2,652 → 2,824 |
| Harmful corrections, of clean words | 14.45% → 14.88% | 13.13% → 13.67% | 12.59% → 13.67% |

The rare words gain on every hand. Their harmful rate rises because it is a rate of the words
tapped cleanly, and more rare words are now tapped cleanly (89, 160 and 172 more), which
autocorrect, as with any clean rare word near a dictionary word, sometimes changes: 29, 36 and 52
more harmful corrections. Counted by word, those committed right with the plain hit test and
wrong with this one are 115, 100 and 121, against 175, 225 and 243 the other way.

### With learning where the user taps (`--limit 5000 --learning`, 2,000 sentences warm-up)

Word accuracy after autocorrect; off is learning off, (a) the aimed decoder input, (a)+(b) that and
the aimed hit test. The left column reproduces the shipped baseline (93.67% off, 94.84% with both).

| User | No resize: off / (a) / (a)+(b) | Resized: off / (a) / (a)+(b) |
|---|---:|---:|
| neutral | 96.90 / 96.90 / 96.90 | 97.68 / 97.68 / 97.68 |
| left thumb | 93.67 / 93.88 / 94.84 | 95.11 / 95.28 / **95.85** |
| left thumb, own aim per key | 93.11 / 93.40 / 94.50 | 94.69 / 94.93 / 95.59 |
| changes grip: first 500 after | 93.61 / 93.82 / 94.20 | 95.08 / 95.22 / 95.32 |
| changes grip: rest | 93.40 / 93.57 / 94.02 | 94.85 / 94.94 / 95.44 |
| right thumb, other aim per key | 93.32 / 93.53 / 94.00 | 94.80 / 94.93 / 95.41 |

| User | Typos, resized: off / (a)+(b) | Harmful, no resize: off / (a)+(b) | Harmful, resized: off / (a)+(b) |
|---|---:|---:|---:|
| neutral | 8.63% / 8.63% | 0.57% / 0.57% | 0.53% / 0.53% |
| left thumb | 20.39% / 20.21% | 0.49% / 0.54% | 0.43% / 0.49% |
| changes grip: first 500 after | 21.50% / 22.44% | 0.20% / 0.22% | 0.16% / 0.18% |
| right thumb, other aim | 21.68% / 21.08% | 0.48% / 0.51% | 0.43% / 0.46% |

The two add up but overlap: the aim alone gains 1.17 points for the left thumb, the odds alone
1.44, both 2.18. Right after a grip change, the stale aim moves a few taps the odds then can't
pull back (typos 21.50% → 22.44%), though accuracy still rises; the aim's decay then catches up.

### Tuning (`--skip 5000 --limit 2000`, all text)

| across, down, cap, floor | Neutral | Left | Right | Left typos | Left harmful | Rare, neutral | Rare, left |
|---|---:|---:|---:|---:|---:|---:|---:|
| off | 97.03% | 93.53% | 93.57% | 37.65% | 0.54% | 66.63% | 46.39% |
| 0.0625, 0.04, 0.10, 0.1 | 97.64% | 94.85% | 94.98% | 23.43% | 0.48% | 67.97% | 49.26% |
| 0.0625, 0.04, 0.12, 0.1 | 97.68% | 94.95% | 95.11% | 21.67% | 0.47% | 67.82% | 49.46% |
| 0.0625, 0.04, 0.15, 0.1 | 97.73% | 95.09% | 95.26% | 19.41% | 0.46% | 67.38% | 49.75% |
| 0.0625, 0.04, 0.20, 0.1 | 97.76% | 95.21% | 95.40% | 16.87% | 0.44% | 67.08% | 49.60% |
| 0.0625, 0.04, 0.15, 0.05 | 97.70% | 95.11% | 95.29% | 19.27% | 0.46% | 67.08% | 50.05% |
| 0.0625, 0.04, 0.15, 0.2 | 97.72% | 95.03% | 95.23% | 19.72% | 0.46% | 67.43% | 49.36% |
| 0.04, 0.025, 0.12, 0.1 | 97.68% | 94.84% | 95.03% | 22.26% | 0.47% | 67.97% | 49.36% |
| **0.04, 0.025, 0.15, 0.1** | **97.73%** | **94.93%** | **95.13%** | **20.49%** | **0.46%** | **67.77%** | **49.55%** |
| no first letter, 0.04, 0.025, 0.15 | 97.53% | 94.35% | 94.64% | 22.80% | 0.47% | 68.12% | 49.46% |
| floor backing off, −0.01 | 97.72% | 95.02% | 95.22% | 19.64% | 0.47% | 67.38% | 49.90% |
| floor backing off, −0.1 | 97.69% | 94.95% | 95.13% | 20.17% | 0.47% | 67.23% | 49.70% |
| floor backing off, −1 | 97.64% | 94.66% | 94.78% | 22.18% | 0.48% | 67.82% | 49.01% |

The shipped setting is the theory's (0.0625, 0.04: the scoreboard's own spread) scaled down. On
the report text the theory's is 0.04–0.16 point more accurate but loses 43% and 37% more words
the plain hit test had right (neutral 427, left 656 against 298 and 477) and its neutral rare
accuracy is lower (65.51% against 65.65%); the shipped one trusts the odds less than the
simulated finger would allow, since a real finger isn't the simulator's. Caps of 0.2 and above
gain a little more overall and cost the neutral typist's rare words. The "backing off" rows
weakened the prior as the typed prefix became unlikely (a negative floor scaled the spread by
the prefix's odds); none beat the plain floor on rare words, and the stronger ones cost overall.
Leaving the first letter of a word alone cost 0.58 point for the left thumb and gained 0.35 on
neutral rare words; the first letter ships.

## Latency

`--threads 1 --limit 250 --correction-only`, on the host Mac, release build, per tap:

| | Mean | p95 |
|---|---:|---:|
| Plain hit test | 0.01 µs | 0.04 µs |
| Letter odds worked out and the resized hit test | 8.37 µs | 23.29 µs |

Within the 0.2 ms budget by an order of magnitude even if the odds were worked out at key down;
on the phone they are worked out on the suggest path after the previous keystroke, and key down
only reads them. The model's per-word running sums (one `f64` per word, about 0.6 MB for the
shipped model) are built once, the first time odds are asked for, so that one-off is not in the
table. Timed on its own (2026-11-16, host Mac, Apple M5 Pro, release build, the shipped model, 3
runs of 30 fresh opens, the first `next_units` call against the second):

| | Min | Median | Max |
|---|---:|---:|---:|
| First call, building the sums | 88.6 µs | 106.5 µs | 126.4 µs |
| A later call | 1.25 µs | 1.4 µs | 5.0 µs |

So the build is about 0.1 ms, far under the 50 ms that would have justified building it ahead
of time or in the background; nothing was changed. Its memory is 77,758 `f64`s (622 KB) plus two
256-entry tables (4 KB), and the process's peak resident memory with 30 models open agrees: 18.9
MB more with their sums built than without, 630 KB each.

## Fast typing

Separate from the resize, the render crate's touch protocol was checked for fast typing
(`crates/render/tests/fast_typing.rs`, glide typing on and off):

* a rolled key goes up, typed, before the next goes down, and its finger lifting types nothing
  more;
* three keys held together type in the order they went down whichever finger lifts first (all six
  orders);
* a second finger landing while the first rocks a little types both, in order;
* 600 taps a millisecond apart, a third of them overlapping one or two others, with small drifts,
  type exactly what was pressed.

One bug was found and fixed: space and delete act when they go up, and neither renderer let go
of a held space or delete when another finger went down, so a letter tapped over a held space
was typed before it ("thec " for "the c") and one tapped over a held delete was deleted by it.
Both now go up, typed, when another finger lands (`input::pointer::rolls`), unless their finger
has swiped (a cursor swipe goes on under the letter) or been held long. Shift and the other keys
a second finger plays chords with stay down. The test transcription of the Compose controller
(`ReferenceRendererTouch`) states the rule on its own, so `RendererTouchMatchTest`'s 1,800 seeded
multi-pointer runs still hold the two renderers to the same actions.
