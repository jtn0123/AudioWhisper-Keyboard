//! `cargo run --release -p scoreboard -- --test NAME=FILE [--test NAME=FILE ...] [options]`
//!
//! Options:
//!   --dict FILE     the dictionary to score (default: the app's main_en-US.dict)
//!   --model FILE    an n-gram model to score with it (default: none)
//!   --typing W      the model's weight on typed candidates (default: the session's)
//!   --swipe L       the least lift the swipe ranker counts (default: the session's)
//!   --limit N       score the first N sentences of each test file (default: all)
//!   --seed N        the seed for taps and swipes (default 20261116)
//!   --threads N     scoring threads (default: the machine's)
//!   --learning      run the tap-aim learning run instead (see `scoreboard::learning`)
//!   --warm-up N     learning run: sentences typed before scoring starts (default 2000)
//!   --from-corrected nothing|unchanged|rebased
//!                   learning run: what a kept auto-correction teaches (default nothing)
//!
//! Prints a markdown table: a column for each test file, then one for all of them; for the
//! learning run, a row for each user and a column for each way of using what was learned.

use scoreboard::geometry::Keyboard;
use scoreboard::learning::FromCorrected;
use scoreboard::report::{Tally, table};
use scoreboard::{Source, text};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Instant;
use suggest::language_model::Tuning;

struct Options {
    dict: PathBuf,
    model: Option<PathBuf>,
    tuning: Tuning,
    tests: Vec<(String, PathBuf)>,
    limit: usize,
    seed: u64,
    threads: usize,
    learning: bool,
    warm_up: usize,
    from_corrected: FromCorrected,
}

fn app() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../app/src/main")
}

fn options() -> Result<Options, String> {
    let mut options = Options {
        dict: app().join("assets/dicts/main_en-US.dict"),
        model: None,
        tuning: Tuning::DEFAULT,
        tests: Vec::new(),
        limit: usize::MAX,
        seed: 20261116,
        threads: std::thread::available_parallelism().map_or(4, |n| n.get()),
        learning: false,
        warm_up: 2000,
        from_corrected: FromCorrected::Nothing,
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or(format!("{flag} needs a value"));
        let number = |v: String| v.parse::<u64>().map_err(|e| format!("{flag}: {e}"));
        let real = |v: String| v.parse::<f32>().map_err(|e| format!("{flag}: {e}"));
        match flag.as_str() {
            "--dict" => options.dict = value()?.into(),
            "--model" => options.model = Some(value()?.into()),
            "--typing" => options.tuning.typing = real(value()?)?,
            "--swipe" => options.tuning.swipe = real(value()?)?,
            "--test" => {
                let v = value()?;
                let (name, file) = v.split_once('=').ok_or("--test takes NAME=FILE")?;
                options.tests.push((name.to_string(), file.into()));
            }
            "--limit" => options.limit = number(value()?)? as usize,
            "--seed" => options.seed = number(value()?)?,
            "--threads" => options.threads = number(value()?)? as usize,
            "--learning" => options.learning = true,
            "--warm-up" => options.warm_up = number(value()?)? as usize,
            "--from-corrected" => {
                options.from_corrected = match value()?.as_str() {
                    "nothing" => FromCorrected::Nothing,
                    "unchanged" => FromCorrected::Unchanged,
                    "rebased" => FromCorrected::Rebased,
                    other => return Err(format!("--from-corrected {other}?")),
                }
            }
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
    let model = match &options.model {
        Some(file) => {
            let bytes = std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
            let model = ngram::Model::open(bytes).ok_or("the model did not open")?;
            Some((Arc::new(model), options.tuning))
        }
        None => None,
    };
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
    if options.learning {
        let sentences: Vec<_> = sources.into_iter().map(|s| s.sentences).collect();
        let learning = scoreboard::learning_run::Options {
            seed: options.seed,
            warm_up: options.warm_up,
            from_corrected: options.from_corrected,
        };
        let table = scoreboard::learning_run::run(
            dictionary.into(),
            model.as_ref(),
            &keyboard,
            &sentences,
            &learning,
        )?;
        print!("{table}");
        eprintln!("scored in {:.1} s", started.elapsed().as_secs_f64());
        return Ok(());
    }
    let tallies = scoreboard::run(
        dictionary.into(),
        model.as_ref(),
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
    if let Some(file) = &options.model {
        println!(
            "Model: {} ({} bytes); {:?}",
            std::fs::canonicalize(file)
                .unwrap_or_else(|_| file.clone())
                .display(),
            std::fs::metadata(file).map_or(0, |m| m.len()),
            options.tuning,
        );
    }
    println!();
    print!("{}", table(&columns));
    eprintln!("scored in {:.1} s", started.elapsed().as_secs_f64());
    Ok(())
}
