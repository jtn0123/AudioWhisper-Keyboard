//! `cargo run --release -p scoreboard -- --test NAME=FILE [--test NAME=FILE ...] [options]`
//!
//! Options:
//!   --dict FILE     the dictionary to score (default: the app's main_en-US.dict)
//!   --model FILE    an n-gram model to score with it (default: none)
//!   --typing W      the model's weight on typed candidates (default: the session's)
//!   --swipe L       the least lift the swipe ranker counts (default: the session's)
//!   --limit N       score the first N sentences of each test file (default: all)
//!   --skip N        ... after skipping its first N (default 0): tune on sentences the results
//!                   don't use
//!   --seed N        the seed for taps and swipes (default 20261116)
//!   --threads N     scoring threads (default: the machine's)
//!   --swipe-only    score swipes only
//!   --strength S    auto-correction strength: off, modest, normal (default) or aggressive
//!   --threshold T   ... or any threshold, for choosing the strengths' (`off` for none)
//!   --correction-only
//!                   score auto-correction only
//!   --resize A,D,C[,F]
//!                   the hit test leans on the letter odds (`keys::resize::Resize`: A across,
//!                   D down, a border moves at most C; F of the odds spread over every letter,
//!                   default `LetterOdds::FLOOR`)
//!
//! How swipes are decoded (default: as the keyboard does; each flag changes one thing):
//!   --original      FlorisBoard's classifier and the flat frequency curve
//!   --ends E        `glide::Tuning::ends`
//!   --dwell D       `glide::Tuning::dwell`
//!   --reach R       `glide::Tuning::reach`
//!   --above A       the frequency curve's cost a step above common (`swipe_words::Curve`)
//!   --history FROM  the user has typed each test file's 5,000 sentences from FROM on before
//!                   (into the learned words, which swipes weigh); keep it clear of the scored ones
//!   --boost B,M     `swipe_words::Boost`: B steps per doubling of a learned word's uses, at most M
//!
//! The tap-aim learning run (see `scoreboard::learning`), instead of the above:
//!   --learning      run it
//!   --warm-up N     sentences typed before scoring starts (default 2000)
//!   --from-corrected nothing|unchanged|rebased
//!                   what a kept auto-correction teaches (default nothing)
//!
//! Prints a markdown table: a column for each test file, then one for all of them; then each
//! swipe variant's counts over all of them. For the learning run, a row for each user and a
//! column for each way of using what was learned.

use keys::resize::Resize;
use scoreboard::geometry::Keyboard;
use scoreboard::learning::FromCorrected;
use scoreboard::report::{Tally, table};
use scoreboard::swipe::{Decoder, VARIANTS};
use scoreboard::touch::KeyResize;
use scoreboard::{Settings, Source, text};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Instant;
use suggest::auto_correction::Strength;
use suggest::language_model::Tuning;
use suggest::letter_odds::LetterOdds;
use suggest::swipe_words::Boost;

struct Options {
    dict: PathBuf,
    model: Option<PathBuf>,
    tuning: Tuning,
    tests: Vec<(String, PathBuf)>,
    limit: usize,
    skip: usize,
    history: Option<usize>,
    settings: Settings,
    learning: bool,
    warm_up: usize,
    from_corrected: FromCorrected,
}

/// How many sentences of each test file `--history` learns from.
const HISTORY: usize = 5000;

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
        skip: 0,
        history: None,
        settings: Settings {
            seed: 20261116,
            threads: std::thread::available_parallelism().map_or(4, |n| n.get()),
            decoder: Decoder::DEFAULT,
            swipe_only: false,
            threshold: Strength::DEFAULT.threshold(),
            correction_only: false,
            resize: None,
        },
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
            "--skip" => options.skip = number(value()?)? as usize,
            "--seed" => options.settings.seed = number(value()?)?,
            "--threads" => options.settings.threads = number(value()?)? as usize,
            "--swipe-only" => options.settings.swipe_only = true,
            "--strength" => {
                let v = value()?.to_ascii_uppercase();
                let strength = Strength::ALL.into_iter().find(|s| s.name() == v);
                options.settings.threshold =
                    strength.ok_or(format!("--strength {v}?"))?.threshold();
            }
            "--threshold" => {
                let v = value()?;
                options.settings.threshold = if v == "off" { None } else { Some(real(v)?) };
            }
            "--correction-only" => options.settings.correction_only = true,
            "--resize" => options.settings.resize = Some(resize(&value()?)?),
            "--original" => options.settings.decoder = Decoder::ORIGINAL,
            "--ends" => options.settings.decoder.glide.ends = real(value()?)?,
            "--dwell" => options.settings.decoder.glide.dwell = real(value()?)?,
            "--reach" => options.settings.decoder.glide.reach = real(value()?)?,
            "--history" => options.history = Some(number(value()?)? as usize),
            "--boost" => {
                let v = value()?;
                let (per, most) = v.split_once(',').ok_or("--boost takes B,M")?;
                let step = |v: &str| v.parse::<i32>().map_err(|e| format!("--boost: {e}"));
                options.settings.decoder.boost = Boost {
                    per_doubling: step(per)?,
                    most: step(most)?,
                };
            }
            "--above" => options.settings.decoder.curve.above = f64::from(real(value()?)?),
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

/// `--resize A,D,C[,F]`.
fn resize(value: &str) -> Result<KeyResize, String> {
    let numbers = value
        .split(',')
        .map(|v| v.parse::<f32>().map_err(|e| format!("--resize: {e}")))
        .collect::<Result<Vec<f32>, String>>()?;
    let (across, down, cap, floor) = match numbers[..] {
        [a, d, c] => (a, d, c, LetterOdds::FLOOR),
        [a, d, c, f] => (a, d, c, f),
        _ => return Err("--resize takes A,D,C[,F]".to_string()),
    };
    let tuning = Resize { across, down, cap };
    Ok(KeyResize { tuning, floor })
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
    let mut history = Vec::new();
    for (name, file) in &options.tests {
        let contents =
            std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
        let mut sentences = text::sentences(&contents);
        if let Some(from) = options.history {
            history.extend(sentences.iter().skip(from).take(HISTORY).cloned());
        }
        sentences.drain(..options.skip.min(sentences.len()));
        sentences.truncate(options.limit);
        sources.push(Source {
            name: name.clone(),
            sentences,
        });
    }
    if options.learning {
        let sentences: Vec<_> = sources.into_iter().map(|s| s.sentences).collect();
        let learning = scoreboard::learning_run::Options {
            seed: options.settings.seed,
            warm_up: options.warm_up,
            from_corrected: options.from_corrected,
            resize: options.settings.resize,
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
        options.settings,
        &history,
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
        "Dictionary: {} ({} bytes); seed {}; {} sentences after {}; threshold {:?}; {:?}; {:?}",
        std::fs::canonicalize(&options.dict)
            .unwrap_or_else(|_| options.dict.clone())
            .display(),
        std::fs::metadata(&options.dict).map_or(0, |m| m.len()),
        options.settings.seed,
        sources.iter().map(|s| s.sentences.len()).sum::<usize>(),
        options.skip,
        options.settings.threshold,
        options.settings.decoder,
        options.settings.resize,
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
    println!();
    for (v, (simulator, hand)) in VARIANTS.iter().enumerate() {
        let swipes = &all.swipe[v];
        println!(
            "swipes {} {}: {} top-1 {} top-3 of {}",
            simulator.name(),
            hand.name(),
            swipes.top1,
            swipes.top3,
            swipes.swipes
        );
    }
    let hands = scoreboard::HANDS.iter().map(|(hand, _)| hand.name());
    for (typist, c) in hands
        .zip(&all.hands)
        .chain([("no apostrophes", &all.no_apostrophes)])
    {
        println!(
            "corrections {typist}: {} words, {} clean, {} right, {} harmful, {} missed, {} wrong fixes",
            c.words, c.clean, c.right, c.harmful, c.missed, c.wrong_fix
        );
    }
    eprintln!("scored in {:.1} s", started.elapsed().as_secs_f64());
    Ok(())
}
