//! Counting results and printing them as a markdown table.

use crate::typing::Corrected;
use std::fmt::Write;
use std::time::Duration;

/// Auto-correction's results for one way of tapping.
#[derive(Clone, Copy, Debug, Default)]
pub struct Corrections {
    pub words: usize,
    /// Words the taps typed as meant.
    pub clean: usize,
    /// Words committed as meant.
    pub right: usize,
    /// Cleanly typed words that auto-correction changed.
    pub harmful: usize,
    /// Typos left as typed.
    pub missed: usize,
    /// Typos changed into another wrong word.
    pub wrong_fix: usize,
    /// Words the plain hit test typed as meant that the hit test as scored did not.
    pub spoiled: usize,
    /// Words the hit test as scored typed as meant that the plain hit test did not.
    pub rescued: usize,
    /// Words committed right with the plain hit test and wrong with the one scored.
    pub lost: usize,
    /// Words committed wrong with the plain hit test and right with the one scored.
    pub won: usize,
}

impl Corrections {
    pub fn add(&mut self, corrected: Corrected) {
        self.words += 1;
        self.clean += usize::from(corrected.clean);
        self.right += usize::from(corrected.right);
        self.harmful += usize::from(corrected.clean && corrected.changed);
        self.missed += usize::from(!corrected.clean && !corrected.changed);
        self.wrong_fix += usize::from(!corrected.clean && corrected.changed && !corrected.right);
        self.spoiled += usize::from(corrected.plain && !corrected.clean);
        self.rescued += usize::from(!corrected.plain && corrected.clean);
        self.lost += usize::from(corrected.plain_right && !corrected.right);
        self.won += usize::from(!corrected.plain_right && corrected.right);
    }

    fn merge(&mut self, other: Corrections) {
        self.words += other.words;
        self.clean += other.clean;
        self.right += other.right;
        self.harmful += other.harmful;
        self.missed += other.missed;
        self.wrong_fix += other.wrong_fix;
        self.spoiled += other.spoiled;
        self.rescued += other.rescued;
        self.lost += other.lost;
        self.won += other.won;
    }

    fn typos(&self) -> usize {
        self.words - self.clean
    }
}

/// Swipes of words a swipe can spell (two keys or more), and how many came out first or in the
/// top 3.
#[derive(Clone, Copy, Debug, Default)]
pub struct Swipes {
    pub swipes: usize,
    pub top1: usize,
    pub top3: usize,
}

impl Swipes {
    /// A swipe whose word came out at `at` of the strip, if at all.
    pub fn add(&mut self, at: Option<usize>) {
        self.swipes += 1;
        self.top1 += usize::from(at == Some(0));
        self.top3 += usize::from(at.is_some());
    }

    fn merge(&mut self, other: Swipes) {
        self.swipes += other.swipes;
        self.top1 += other.top1;
        self.top3 += other.top3;
    }
}

/// Everything counted over a set of sentences.
#[derive(Clone, Debug, Default)]
pub struct Tally {
    pub words: usize,
    pub next_top1: usize,
    pub next_top3: usize,
    pub letters: usize,
    pub keystrokes: usize,
    /// Auto-correction for each of [`crate::HANDS`]: neutral, left thumb, right thumb.
    pub hands: [Corrections; 3],
    /// The same for the words the n-gram model doesn't have, and those the dictionary doesn't.
    pub outside_model: [Corrections; 3],
    pub outside_dictionary: [Corrections; 3],
    /// Every key hit, but no apostrophe typed: the words with one are its "typos".
    pub no_apostrophes: Corrections,
    /// Swipes, for each of [`crate::swipe::VARIANTS`].
    pub swipe: [Swipes; 4],
    /// How long each swipe took to decode.
    pub swipe_time: Vec<Duration>,
    /// How long each strip took while typing a word.
    pub typing_time: Vec<Duration>,
    /// How long each strip of next words took after a space.
    pub prediction_time: Vec<Duration>,
    /// How long each tap's key took to find with the letter odds.
    pub hit_time: Vec<Duration>,
}

impl Tally {
    pub fn add(&mut self, other: Tally) {
        self.words += other.words;
        self.next_top1 += other.next_top1;
        self.next_top3 += other.next_top3;
        self.letters += other.letters;
        self.keystrokes += other.keystrokes;
        for (mine, theirs) in [
            (&mut self.hands, other.hands),
            (&mut self.outside_model, other.outside_model),
            (&mut self.outside_dictionary, other.outside_dictionary),
        ] {
            for (mine, theirs) in mine.iter_mut().zip(theirs) {
                mine.merge(theirs);
            }
        }
        self.no_apostrophes.merge(other.no_apostrophes);
        for (mine, theirs) in self.swipe.iter_mut().zip(other.swipe) {
            mine.merge(theirs);
        }
        self.swipe_time.extend(other.swipe_time);
        self.typing_time.extend(other.typing_time);
        self.prediction_time.extend(other.prediction_time);
        self.hit_time.extend(other.hit_time);
    }

    /// The mean and 95th percentile of [`Tally::typing_time`], in milliseconds.
    pub fn latency(&self) -> (f64, f64) {
        latency(&self.typing_time)
    }

    /// The mean and 95th percentile of [`Tally::swipe_time`], in milliseconds.
    pub fn swipe_latency(&self) -> (f64, f64) {
        latency(&self.swipe_time)
    }

    /// The mean and 95th percentile of [`Tally::prediction_time`], in milliseconds.
    pub fn prediction_latency(&self) -> (f64, f64) {
        latency(&self.prediction_time)
    }

    /// The mean and 95th percentile of [`Tally::hit_time`], in milliseconds.
    pub fn hit_latency(&self) -> (f64, f64) {
        latency(&self.hit_time)
    }
}

fn latency(times: &[Duration]) -> (f64, f64) {
    let mut times: Vec<f64> = times.iter().map(|t| t.as_secs_f64() * 1e3).collect();
    if times.is_empty() {
        return (0.0, 0.0);
    }
    times.sort_by(f64::total_cmp);
    let mean = times.iter().sum::<f64>() / times.len() as f64;
    (mean, times[(times.len() * 95 / 100).min(times.len() - 1)])
}

fn percent(part: usize, whole: usize) -> String {
    if whole == 0 {
        "-".to_string()
    } else {
        format!("{:.1}%", 100.0 * part as f64 / whole as f64)
    }
}

/// A share to two places: corrections are compared a word at a time.
fn fine(part: usize, whole: usize) -> String {
    if whole == 0 {
        "-".to_string()
    } else {
        format!("{:.2}%", 100.0 * part as f64 / whole as f64)
    }
}

type Row = (String, Box<dyn Fn(&Tally) -> String>);

/// Variant `v`'s swipe top-1, or top-3.
fn swipe(t: &Tally, v: usize, top1: bool) -> String {
    let s = &t.swipe[v];
    percent(if top1 { s.top1 } else { s.top3 }, s.swipes)
}

fn row(label: &str, value: impl Fn(&Tally) -> String + 'static) -> Row {
    (label.to_string(), Box::new(value))
}

/// Auto-correction's rows for each hand, of `which` words.
fn correction_rows(rows: &mut Vec<Row>, words: &str, which: fn(&Tally) -> &[Corrections; 3]) {
    const HANDS: [&str; 3] = ["neutral", "left thumb", "right thumb"];
    type Measure = fn(&Corrections) -> String;
    let measures: [(&str, Measure); 11] = [
        ("Typos made", |c| fine(c.typos(), c.words)),
        ("Word accuracy after autocorrect", |c| {
            fine(c.right, c.words)
        }),
        ("Harmful corrections", |c| fine(c.harmful, c.clean)),
        ("Missed typos", |c| fine(c.missed, c.typos())),
        ("Wrong fixes", |c| fine(c.wrong_fix, c.typos())),
        ("Words committed right", |c| c.right.to_string()),
        ("Words tapped cleanly", |c| c.clean.to_string()),
        ("Words typed right only by the plain hit test", |c| {
            c.spoiled.to_string()
        }),
        ("Words typed right only by this hit test", |c| {
            c.rescued.to_string()
        }),
        ("Words committed right only with the plain hit test", |c| {
            c.lost.to_string()
        }),
        ("Words committed right only with this hit test", |c| {
            c.won.to_string()
        }),
    ];
    for (h, hand) in HANDS.into_iter().enumerate() {
        for (name, measure) in measures {
            let of = match name {
                "Harmful corrections" => ", of clean words",
                "Missed typos" | "Wrong fixes" => ", of typos",
                _ => "",
            };
            let label = format!("{name} ({hand}{words}{of})");
            rows.push(row(&label, move |t| measure(&which(t)[h])));
        }
    }
}

fn rows() -> Vec<Row> {
    let mut rows = vec![
        row("Words scored", |t| t.words.to_string()),
        row("Next word top-1", |t| percent(t.next_top1, t.words)),
        row("Next word top-3", |t| percent(t.next_top3, t.words)),
        row("Keystroke savings", |t| {
            percent(t.letters - t.keystrokes, t.letters)
        }),
    ];
    correction_rows(&mut rows, "", |t| &t.hands);
    rows.push(row("Rare words: outside the model", |t| {
        t.outside_model[0].words.to_string()
    }));
    correction_rows(&mut rows, ", outside the model", |t| &t.outside_model);
    rows.push(row("Rare words: outside the dictionary", |t| {
        t.outside_dictionary[0].words.to_string()
    }));
    correction_rows(&mut rows, ", outside the dictionary", |t| {
        &t.outside_dictionary
    });
    rows.extend([
        row("Words with an apostrophe", |t| {
            t.no_apostrophes.typos().to_string()
        }),
        row("Word accuracy after autocorrect (no apostrophes)", |t| {
            percent(t.no_apostrophes.right, t.no_apostrophes.words)
        }),
        row("Apostrophes put back (of words with one)", |t| {
            let a = &t.no_apostrophes;
            percent(a.typos() - a.missed - a.wrong_fix, a.typos())
        }),
        row(
            "Harmful corrections (no apostrophes, of words without one)",
            |t| percent(t.no_apostrophes.harmful, t.no_apostrophes.clean),
        ),
        row("Left without (no apostrophes, of words with one)", |t| {
            percent(t.no_apostrophes.missed, t.no_apostrophes.typos())
        }),
        row("Wrong fixes (no apostrophes, of words with one)", |t| {
            percent(t.no_apostrophes.wrong_fix, t.no_apostrophes.typos())
        }),
        row("Swipe words", |t| t.swipe[0].swipes.to_string()),
        row("Swipe top-1 (trace, neutral)", |t| swipe(t, 0, true)),
        row("Swipe top-3 (trace, neutral)", |t| swipe(t, 0, false)),
        row("Swipe top-1 (trace, left thumb)", |t| swipe(t, 1, true)),
        row("Swipe top-3 (trace, left thumb)", |t| swipe(t, 1, false)),
        row("Swipe top-1 (lognormal, neutral)", |t| swipe(t, 2, true)),
        row("Swipe top-3 (lognormal, neutral)", |t| swipe(t, 2, false)),
        row("Swipe top-1 (lognormal, left thumb)", |t| swipe(t, 3, true)),
        row("Swipe top-3 (lognormal, left thumb)", |t| {
            swipe(t, 3, false)
        }),
        row("Swipe decode per gesture, mean (ms)", |t| {
            format!("{:.2}", t.swipe_latency().0)
        }),
        row("Swipe decode per gesture, p95 (ms)", |t| {
            format!("{:.2}", t.swipe_latency().1)
        }),
        row("Strip time per keystroke, mean (ms)", |t| {
            format!("{:.2}", t.latency().0)
        }),
        row("Strip time per keystroke, p95 (ms)", |t| {
            format!("{:.2}", t.latency().1)
        }),
        row("Next-word strip time, mean (ms)", |t| {
            format!("{:.2}", t.prediction_latency().0)
        }),
        row("Next-word strip time, p95 (ms)", |t| {
            format!("{:.2}", t.prediction_latency().1)
        }),
        row(
            "Hit test per tap, with the letter odds if on, mean (µs)",
            |t| format!("{:.2}", 1000.0 * t.hit_latency().0),
        ),
        row(
            "Hit test per tap, with the letter odds if on, p95 (µs)",
            |t| format!("{:.2}", 1000.0 * t.hit_latency().1),
        ),
    ]);
    rows
}

/// A markdown table with a column for each named tally.
pub fn table(columns: &[(&str, &Tally)]) -> String {
    let mut out = String::from("| Metric |");
    for (name, _) in columns {
        let _ = write!(out, " {name} |");
    }
    out.push_str("\n|---|");
    out.push_str(&"---:|".repeat(columns.len()));
    out.push('\n');
    for (label, value) in rows() {
        let _ = write!(out, "| {label} |");
        for (_, tally) in columns {
            let _ = write!(out, " {} |", value(tally));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_harm_misses_and_wrong_fixes() {
        let mut c = Corrections::default();
        let word = |clean, changed, right| Corrected {
            clean,
            changed,
            right,
            plain: clean,
            plain_right: right,
        };
        c.add(word(true, false, true));
        c.add(word(true, true, false));
        c.add(word(false, false, false));
        c.add(word(false, true, true));
        c.add(word(false, true, false));
        assert_eq!((c.words, c.clean, c.right), (5, 2, 2));
        assert_eq!((c.harmful, c.missed, c.wrong_fix, c.typos()), (1, 1, 1, 3));
        assert_eq!((c.spoiled, c.rescued), (0, 0));
        c.add(Corrected {
            plain: false,
            ..word(true, false, true)
        });
        c.add(Corrected {
            plain: true,
            plain_right: true,
            ..word(false, false, false)
        });
        assert_eq!((c.spoiled, c.rescued), (1, 1));
        assert_eq!((c.lost, c.won), (1, 0));
        c.add(Corrected {
            plain_right: false,
            ..word(true, false, true)
        });
        assert_eq!((c.lost, c.won), (1, 1));
    }

    #[test]
    fn prints_one_row_per_metric() {
        let tally = Tally {
            words: 4,
            next_top1: 1,
            next_top3: 2,
            ..Tally::default()
        };
        let table = table(&[("all", &tally)]);
        assert!(table.contains("| Next word top-1 | 25.0% |"));
        assert_eq!(table.lines().count(), 2 + rows().len());
    }
}
