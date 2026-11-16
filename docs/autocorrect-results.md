# Auto-correction: results

What the scoreboard (`lib/rust/crates/scoreboard`) measured for the auto-correction changes of
2026-11-16. Why each was kept, and what would change it, is in [decisions.md](decisions.md).

Every run uses the app's dictionary and n-gram model and the held-out text of
`build/language-model/run-20261116-051542/corpus` (Tatoeba and Global Voices test files). Choices
were made on sentences 5,001–7,000 of each file (`--skip 5000 --limit 2000`); the report rows are
the first 5,000 of each (`--limit 5000`, 124,547 words), which no choice looked at.

```
cargo run --release -p scoreboard -- \
  --model ../../app/src/main/assets/dicts/main_en-US.lm \
  --test tatoeba=$CORPUS/test-tatoeba.txt --test globalvoices=$CORPUS/test-globalvoices.txt \
  --limit 5000 --strength normal          # or off, modest, aggressive; --threshold T to sweep
```

The rows: *word accuracy* is words right after auto-correction; *harmful* is correct words the
keyboard changed (of words typed correctly); *missed* is typos left as typed (of typos); *wrong
fixes* is typos changed to another wrong word (of typos). The neutral typist misses a key's centre
evenly; the left thumb is the user's grip, which misses more.

## Auto-correction strength

*Typing → Corrections → Auto-correction*: Off, Modest, Normal (default) or Aggressive. Each is a
threshold the best correction's normalized score must reach (`suggest::auto_correction::Strength`);
Normal is 0.185, AOSP's "modest" value, which the keyboard always used, so the default changes
nothing (its rows below are the language-model report's, to the digit).

### Choosing the thresholds (tuning sentences, both files)

| Threshold | Accuracy (neutral) | Harmful | Missed | Wrong fixes | Accuracy (left thumb) | Harmful | Missed | Wrong fixes |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| off | 80.2% | 0.0% | 100.0% | 0.0% | 62.3% | 0.0% | 100.0% | 0.0% |
| 0.60 | 80.5% | 0.1% | 97.5% | 1.0% | 62.8% | 0.1% | 97.9% | 0.8% |
| 0.45 | 85.9% | 0.1% | 69.7% | 1.1% | 69.6% | 0.1% | 79.8% | 0.9% |
| 0.35 | 93.4% | 0.1% | 31.7% | 1.2% | 82.6% | 0.1% | 45.1% | 0.9% |
| 0.30 | 95.6% | 0.2% | 20.4% | 1.3% | 88.1% | 0.1% | 30.3% | 1.1% |
| 0.28 | 96.2% | 0.2% | 17.3% | 1.4% | 89.6% | 0.2% | 26.1% | 1.2% |
| **0.25** | 96.7% | **0.3%** | 14.2% | 1.5% | 91.5% | **0.2%** | 21.0% | 1.3% |
| 0.22 | 97.0% | 0.4% | 12.2% | 1.6% | 92.7% | 0.4% | 17.4% | 1.5% |
| 0.185 (today) | 97.0% | 0.6% | 11.0% | 1.7% | 93.5% | 0.5% | 14.7% | 1.6% |
| 0.15 | 97.0% | 0.8% | 10.2% | 1.9% | 94.1% | 0.7% | 12.7% | 1.9% |
| **0.12** | 96.9% | 1.0% | 9.8% | 2.1% | **94.3%** | 0.9% | 11.8% | 2.1% |
| 0.10 | 96.7% | 1.2% | 9.6% | 2.3% | 94.3% | 1.0% | 11.3% | 2.2% |
| 0.09 | 96.7% | 1.2% | 9.5% | 2.4% | 94.3% | 1.1% | 11.1% | 2.3% |
| 0.067 (AOSP aggressive) | 96.6% | 1.4% | 9.3% | 2.5% | 94.2% | 1.3% | 10.8% | 2.5% |
| 0 | 95.2% | 3.1% | 6.2% | 5.6% | 93.5% | 2.7% | 7.1% | 5.8% |

* **Modest = 0.25**: the lowest threshold that halves harmful corrections for both typists
  (0.6% → 0.3%, 0.5% → 0.2%). It costs 0.3 point of neutral accuracy and 2 of a left thumb's,
  because more typos are left. 0.22 cuts harm by only a third; 0.30 doubles the typos left.
* **Aggressive = 0.12**: the highest threshold at which a left thumb's accuracy peaks (94.3%);
  lower adds harmful corrections and fixes nothing more. AOSP's own aggressive (0.067) is worse
  than 0.12 on every row, so it is not used.
* Normal stays the best neutral accuracy, which is why it stays the default.

### Report (first 5,000 sentences of each file)

| Strength | Accuracy (neutral) | Harmful | Missed | Wrong fixes | Accuracy (left thumb) | Harmful | Missed | Wrong fixes |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Off | 79.9% | 0.0% | 100.0% | 0.0% | 62.1% | 0.0% | 100.0% | 0.0% |
| Modest (0.25) | 96.6% | 0.3% | 14.5% | 1.5% | 91.6% | 0.2% | 20.6% | 1.3% |
| **Normal (0.185)** | **96.9%** | 0.6% | 11.3% | 1.8% | 93.7% | 0.5% | 14.2% | 1.6% |
| Aggressive (0.12) | 96.7% | 1.0% | 10.1% | 2.2% | **94.3%** | 0.9% | 11.6% | 2.0% |

By file (accuracy, neutral / left thumb): Tatoeba 82.0/65.2, 97.1/92.4, 97.5/94.2, 97.3/94.8;
Global Voices 78.9/60.8, 96.3/91.2, 96.7/93.5, 96.5/94.1 (Off, Modest, Normal, Aggressive).

Off still shows suggestions; it changes nothing on its own, not even the whitelist ("im" →
"I'm") or a personal word's capitals. The system spell checker keeps Normal's confidence whatever
the setting, since it is not the keyboard's auto-correction.

## Apostrophes

A typist who hits every key but never types an apostrophe ("dont", "im", "its" for "it's") is the
scoreboard's third: its "typos" are the words that have one (2,953 of the report's 124,547), and
its harmful corrections are words without one that it typed right and the keyboard changed.

What AOSP's port already did: a contraction whose letters spell no word ("dont", "im", "youre",
"theyre", "cant", "didnt") is put right, by the decoder or the whitelist. Where the letters spell
a word too, the dictionary can't tell which was meant, so it picked by kind: a valid word is never
corrected ("its", "were", "well", "shed" stayed, even in "I think its"), and a whitelisted one
always is ("he is ill" became "he is I'll", "he lets" "he let's").

Now the language model decides those, from the words before (`suggest::apostrophes`): if it finds
one spelling at least 1.5 nats (4.5 times) likelier than the other, that one is used, whitelisted
or not; otherwise, or if it lacks either word, AOSP's choice stands. Both spellings are asked about
as shown, so a capital keeps a name a name ("United Nations").

### Choosing the margin (tuning sentences, both files)

Counts, of 1,102 words with an apostrophe and 48,091 without for the typist without apostrophes,
and of 39,446 and 30,671 words typed right for the neutral typist and the left thumb.

| Margin (nats) | Apostrophes put back | Harmful (no apostrophes) | Harmful (neutral) | Harmful (left thumb) |
|---|---:|---:|---:|---:|
| none (today) | 797 (72.3%) | 445 | 234 | 167 |
| 1.0 | 949 (86.1%) | 456 | 241 | 171 |
| **1.5** | 943 (85.6%) | 453 | 240 | 170 |
| 2.0 | 924 (83.8%) | 451 | 239 | 169 |
| 3.0 | 914 (82.9%) | 449 | 238 | 169 |

1.5 is the knee: three fewer harmful corrections than 1.0 for six fewer apostrophes put back,
where 2.0 gives up nineteen more to save two.

### Report (first 5,000 sentences of each file)

| | Today | With the model deciding |
|---|---:|---:|
| Apostrophes put back (of 2,953) | 2,170 (73.5%) | **2,527 (85.6%)** |
| … Tatoeba / Global Voices | 84.4% / 56.3% | 95.2% / 70.5% |
| Left without | 741 (25.1%) | 384 (13.0%) |
| Word accuracy, typist without apostrophes | 98.5% | 98.7% |
| Harmful, typist without apostrophes (of 121,594 typed right) | 1,171 | 1,181 |
| Harmful, neutral typist (of 99,461 typed right) | 573 | 579 |
| Harmful, left thumb (of 77,359 typed right) | 372 | 376 |

So 357 more apostrophes are put back for 10 more harmful corrections on clean text; for typists who
do type apostrophes, 6 and 4 more in about 100,000 and 77,000 words, which leaves every rounded
row of the strength table above unchanged. The new harm is mostly a plural read as a possessive
("governments", "islands", "Peoples" after words that usually take one) and an "its" read as
"it's" (one plain "its" in 57 on the tuning sentences). What is still left without is mostly possessives ("Toms", "Marys") the model
has no evidence for, which is why Global Voices, with more names, gains less.

In context (the shipped dictionary and model, `crates/suggest/tests/apostrophes.rs`): "he is
ill", "he lets", "the dog wagged its", "we were", "very well", "the shell", "go to hell" and "the
shed" stay; "I think" followed by "ill", "its", "were", "well", "shed" or "hell" becomes "I'll",
"it's", "we're", "we'll", "she'd" or "he'll", and "Lets" and "Its" starting a sentence become
"Let's" and "It's". Known misses: "we wed" becomes "we'd", the model knowing neither pair and
"we'd" being the commoner word; "my id" still becomes "I'd", as it always did, the model not being
sure enough to overrule the whitelist.
