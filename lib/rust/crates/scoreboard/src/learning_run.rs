//! Runs every user of [`crate::learning`] every way at once, a thread each, and prints the
//! results as markdown tables.

use crate::geometry::Keyboard;
use crate::learning::{self, FromCorrected, Mode, Run};
use crate::report::Corrections;
use crate::text::Sentence;
use crate::typing::WithModel;
use std::fmt::Write;
use std::sync::Arc;

/// The learning run's settings.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub seed: u64,
    /// Sentences typed before scoring starts.
    pub warm_up: usize,
    pub from_corrected: FromCorrected,
}

/// Seeds each user's taps apart from the main scoreboard's.
const LEARNING_STREAM: u64 = 0x7A9_0000;

/// The results of one user typed every way: for each window, one result per [`Mode`].
struct UserResults {
    name: &'static str,
    windows: Vec<(&'static str, usize, Vec<Corrections>)>,
}

pub fn run(
    dictionary: Arc<[u8]>,
    model: Option<&WithModel>,
    keyboard: &Keyboard,
    sources: &[Vec<Sentence>],
    options: &Options,
) -> Result<String, String> {
    let stream = learning::stream(sources);
    let users = learning::users();
    let results = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..)
            .zip(users)
            .flat_map(|(u, user)| Mode::ALL.map(move |mode| (u, user, mode)))
            .map(|(u, user, mode)| {
                let windows = user.windows(stream.len(), options.warm_up);
                let run = Run {
                    keyboard,
                    stream: &stream,
                    user,
                    mode,
                    from_corrected: options.from_corrected,
                    seed: options.seed ^ LEARNING_STREAM ^ u,
                    windows: windows.into_iter().map(|(_, range)| range).collect(),
                };
                let dictionary = dictionary.clone();
                scope.spawn(move || run.score(dictionary, model))
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().map_err(|_| "a learning run panicked"))
            .collect::<Result<Vec<_>, _>>()
    })?;
    let mut results = results.into_iter();
    let users: Vec<UserResults> = users
        .iter()
        .map(|user| {
            let by_mode: Vec<Vec<Corrections>> = results.by_ref().take(Mode::ALL.len()).collect();
            let windows = user
                .windows(stream.len(), options.warm_up)
                .into_iter()
                .enumerate()
                .map(|(w, (label, range))| {
                    let modes = by_mode.iter().map(|windows| windows[w]).collect();
                    (label, range.len(), modes)
                })
                .collect();
            UserResults {
                name: user.name,
                windows,
            }
        })
        .collect();
    let mut out = format!(
        "{} sentences, the first {} not scored; seed {}; kept corrections teach: {:?}\n",
        stream.len(),
        options.warm_up,
        options.seed,
        options.from_corrected
    );
    for (title, value) in metrics() {
        let _ = writeln!(out, "\n{title}\n");
        out.push_str(&table(&users, value));
    }
    Ok(out)
}

type Metric = (&'static str, fn(&Corrections) -> String);

fn share(part: usize, whole: usize) -> String {
    format!("{:.2}%", 100.0 * part as f64 / whole.max(1) as f64)
}

fn metrics() -> [Metric; 6] {
    [
        ("Word accuracy after autocorrect", |c| {
            share(c.right, c.words)
        }),
        ("Typos made", |c| share(c.words - c.clean, c.words)),
        ("Harmful corrections, of clean words", |c| {
            share(c.harmful, c.clean)
        }),
        ("Missed typos, of typos", |c| {
            share(c.missed, c.words - c.clean)
        }),
        ("Words committed right", |c| c.right.to_string()),
        ("Words tapped cleanly", |c| c.clean.to_string()),
    ]
}

fn table(users: &[UserResults], value: fn(&Corrections) -> String) -> String {
    let mut out = String::from("| User | Sentences | Words |");
    for mode in Mode::ALL {
        let _ = write!(out, " {} |", mode.name());
    }
    out.push_str("\n|---|---|---:|");
    out.push_str(&"---:|".repeat(Mode::ALL.len()));
    out.push('\n');
    for user in users {
        for (label, sentences, modes) in &user.windows {
            let name = if user.windows.len() > 1 {
                format!("{}: {label}", user.name)
            } else {
                user.name.to_string()
            };
            let words = modes.first().map_or(0, |c| c.words);
            let _ = write!(out, "| {name} | {sentences} | {words} |");
            for corrections in modes {
                let _ = write!(out, " {} |", value(corrections));
            }
            out.push('\n');
        }
    }
    out
}
