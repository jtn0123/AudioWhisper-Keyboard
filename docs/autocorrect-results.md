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
