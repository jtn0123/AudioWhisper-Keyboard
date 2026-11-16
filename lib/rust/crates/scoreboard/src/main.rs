//! `cargo run --release -p scoreboard -- --test NAME=FILE [--test NAME=FILE ...] [options]`
//!
//! Options:
//!   --dict FILE     the dictionary to score (default: the app's main_en-US.dict)
//!   --limit N       score the first N sentences of each test file (default: all)
//!   --seed N        the seed for taps and swipes (default 20261116)
//!   --threads N     scoring threads (default: the machine's)
//!
//! Prints a markdown table: a column for each test file, then one for all of them.

use scoreboard::geometry::Keyboard;
use scoreboard::report::{Tally, table};
use scoreboard::{Source, text};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

struct Options {
    dict: PathBuf,
    tests: Vec<(String, PathBuf)>,
    limit: usize,
    seed: u64,
    threads: usize,
}

fn app() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../app/src/main")
}

fn options() -> Result<Options, String> {
    let mut options = Options {
        dict: app().join("assets/dicts/main_en-US.dict"),
        tests: Vec::new(),
        limit: usize::MAX,
        seed: 20261116,
        threads: std::thread::available_parallelism().map_or(4, |n| n.get()),
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or(format!("{flag} needs a value"));
        let number = |v: String| v.parse::<u64>().map_err(|e| format!("{flag}: {e}"));
        match flag.as_str() {
            "--dict" => options.dict = value()?.into(),
            "--test" => {
                let v = value()?;
                let (name, file) = v.split_once('=').ok_or("--test takes NAME=FILE")?;
                options.tests.push((name.to_string(), file.into()));
            }
            "--limit" => options.limit = number(value()?)? as usize,
            "--seed" => options.seed = number(value()?)?,
            "--threads" => options.threads = number(value()?)? as usize,
            _ => return Err(format!("unknown option {flag}")),
        }
    }
    if options.tests.is_empty() {
        return Err("give at least one --test NAME=FILE".to_string());
    }
    Ok(options)
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("scoreboard: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let options = options()?;
    let started = Instant::now();
    let dictionary =
        std::fs::read(&options.dict).map_err(|e| format!("{}: {e}", options.dict.display()))?;
    let keyboard = Keyboard::phone(&app().join("assets"))?;
    let mut sources = Vec::new();
    for (name, file) in &options.tests {
        let contents =
            std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
        let mut sentences = text::sentences(&contents);
        sentences.truncate(options.limit);
        sources.push(Source {
            name: name.clone(),
            sentences,
        });
    }
    let tallies = scoreboard::run(
        dictionary.into(),
        &keyboard,
        &sources,
        options.seed,
        options.threads,
    )?;
    let mut all = Tally::default();
    for tally in &tallies {
        all.add(tally.clone());
    }
    let mut columns: Vec<(&str, &Tally)> = sources
        .iter()
        .map(|source| source.name.as_str())
        .zip(&tallies)
        .collect();
    if columns.len() > 1 {
        columns.push(("all", &all));
    }
    println!(
        "Dictionary: {} ({} bytes); seed {}; {} sentences",
        std::fs::canonicalize(&options.dict)
            .unwrap_or_else(|_| options.dict.clone())
            .display(),
        std::fs::metadata(&options.dict).map_or(0, |m| m.len()),
        options.seed,
        sources.iter().map(|s| s.sentences.len()).sum::<usize>(),
    );
    println!();
    print!("{}", table(&columns));
    eprintln!("scored in {:.1} s", started.elapsed().as_secs_f64());
    Ok(())
}
