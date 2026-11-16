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
}

impl Corrections {
    pub fn add(&mut self, corrected: Corrected) {
        self.words += 1;
        self.clean += usize::from(corrected.clean);
        self.right += usize::from(corrected.right);
        self.harmful += usize::from(corrected.clean && corrected.changed);
        self.missed += usize::from(!corrected.clean && !corrected.changed);
        self.wrong_fix += usize::from(!corrected.clean && corrected.changed && !corrected.right);
    }

    fn merge(&mut self, other: Corrections) {
        self.words += other.words;
        self.clean += other.clean;
        self.right += other.right;
        self.harmful += other.harmful;
        self.missed += other.missed;
        self.wrong_fix += other.wrong_fix;
    }

    fn typos(&self) -> usize {
        self.words - self.clean
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
    pub neutral: Corrections,
    pub left_thumb: Corrections,
    /// Words a swipe can spell (two keys or more), and how many came out first or in the top 3.
    pub swipes: usize,
    pub swipe_top1: usize,
    pub swipe_top3: usize,
    /// How long each strip took while typing a word.
    pub typing_time: Vec<Duration>,
    /// How long each strip of next words took after a space.
    pub prediction_time: Vec<Duration>,
}

impl Tally {
    pub fn add(&mut self, other: Tally) {
        self.words += other.words;
        self.next_top1 += other.next_top1;
        self.next_top3 += other.next_top3;
        self.letters += other.letters;
        self.keystrokes += other.keystrokes;
        self.neutral.merge(other.neutral);
        self.left_thumb.merge(other.left_thumb);
        self.swipes += other.swipes;
        self.swipe_top1 += other.swipe_top1;
        self.swipe_top3 += other.swipe_top3;
        self.typing_time.extend(other.typing_time);
        self.prediction_time.extend(other.prediction_time);
    }

    /// The mean and 95th percentile of [`Tally::typing_time`], in milliseconds.
    pub fn latency(&self) -> (f64, f64) {
        latency(&self.typing_time)
    }

    /// The mean and 95th percentile of [`Tally::prediction_time`], in milliseconds.
    pub fn prediction_latency(&self) -> (f64, f64) {
        latency(&self.prediction_time)
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

type Row = (&'static str, fn(&Tally) -> String);

fn rows() -> Vec<Row> {
    vec![
        ("Words scored", |t| t.words.to_string()),
        ("Next word top-1", |t| percent(t.next_top1, t.words)),
        ("Next word top-3", |t| percent(t.next_top3, t.words)),
        ("Keystroke savings", |t| {
            percent(t.letters - t.keystrokes, t.letters)
        }),
        ("Typos made (neutral)", |t| {
            percent(t.neutral.typos(), t.neutral.words)
        }),
        ("Word accuracy after autocorrect (neutral)", |t| {
            percent(t.neutral.right, t.neutral.words)
        }),
        ("Harmful corrections (neutral, of clean words)", |t| {
            percent(t.neutral.harmful, t.neutral.clean)
        }),
        ("Missed typos (neutral, of typos)", |t| {
            percent(t.neutral.missed, t.neutral.typos())
        }),
        ("Wrong fixes (neutral, of typos)", |t| {
            percent(t.neutral.wrong_fix, t.neutral.typos())
        }),
        ("Typos made (left thumb)", |t| {
            percent(t.left_thumb.typos(), t.left_thumb.words)
        }),
        ("Word accuracy after autocorrect (left thumb)", |t| {
            percent(t.left_thumb.right, t.left_thumb.words)
        }),
        ("Harmful corrections (left thumb, of clean words)", |t| {
            percent(t.left_thumb.harmful, t.left_thumb.clean)
        }),
        ("Missed typos (left thumb, of typos)", |t| {
            percent(t.left_thumb.missed, t.left_thumb.typos())
        }),
        ("Wrong fixes (left thumb, of typos)", |t| {
            percent(t.left_thumb.wrong_fix, t.left_thumb.typos())
        }),
        ("Swipe words", |t| t.swipes.to_string()),
        ("Swipe top-1", |t| percent(t.swipe_top1, t.swipes)),
        ("Swipe top-3", |t| percent(t.swipe_top3, t.swipes)),
        ("Strip time per keystroke, mean (ms)", |t| {
            format!("{:.2}", t.latency().0)
        }),
        ("Strip time per keystroke, p95 (ms)", |t| {
            format!("{:.2}", t.latency().1)
        }),
        ("Next-word strip time, mean (ms)", |t| {
            format!("{:.2}", t.prediction_latency().0)
        }),
        ("Next-word strip time, p95 (ms)", |t| {
            format!("{:.2}", t.prediction_latency().1)
        }),
    ]
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
        };
        c.add(word(true, false, true));
        c.add(word(true, true, false));
        c.add(word(false, false, false));
        c.add(word(false, true, true));
        c.add(word(false, true, false));
        assert_eq!((c.words, c.clean, c.right), (5, 2, 2));
        assert_eq!((c.harmful, c.missed, c.wrong_fix, c.typos()), (1, 1, 1, 3));
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
