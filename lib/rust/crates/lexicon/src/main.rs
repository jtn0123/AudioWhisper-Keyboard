//! `lexicon prepare OUT NAME=FILE [NAME=FILE ...]`: splits each source (one sentence a line) into
//! OUT/train-NAME.txt and OUT/test-NAME.txt (see `lexicon::corpus`).
//!
//! `lexicon dictionary OLD OUT TRAIN [TRAIN ...] [--next N,N] [--min-pair N] [--date S]`: counts
//! the training sentences and writes OLD rebuilt from them, once for each `--next` value, to
//! OUT/next-N.dict (see `lexicon::dictionary`).

use lexicon::{corpus, count, dictionary, v202};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: lexicon prepare OUT NAME=FILE [NAME=FILE ...]\n       \
    lexicon dictionary OLD OUT TRAIN [TRAIN ...] [--next N,N] [--min-pair N] [--date S]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("prepare") => prepare(&args[1..]),
        Some("dictionary") => build_dictionary(&args[1..]),
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("lexicon: {error}");
            ExitCode::FAILURE
        }
    }
}

fn sources(args: &[String]) -> Result<Vec<(String, PathBuf)>, String> {
    args.iter()
        .map(|arg| {
            let (name, file) = arg
                .split_once('=')
                .ok_or(format!("{arg}: expected NAME=FILE"))?;
            Ok((name.to_string(), PathBuf::from(file)))
        })
        .collect()
}

fn read(file: &Path) -> Result<String, String> {
    let bytes = std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn write(file: &Path, lines: &[String]) -> Result<(), String> {
    let mut text = lines.join("\n");
    text.push('\n');
    std::fs::write(file, text).map_err(|e| format!("{}: {e}", file.display()))
}

fn prepare(args: &[String]) -> Result<(), String> {
    let (out, rest) = args.split_first().ok_or("prepare needs OUT")?;
    let out = Path::new(out);
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let mut seen = HashSet::new();
    for (name, file) in sources(rest)? {
        let text = read(&file)?;
        let split = corpus::split(text.lines(), &mut seen);
        write(&out.join(format!("train-{name}.txt")), &split.train)?;
        write(&out.join(format!("test-{name}.txt")), &split.test)?;
        println!(
            "{name}: {} training sentences, {} held out, {} dropped",
            split.train.len(),
            split.test.len(),
            split.dropped
        );
    }
    Ok(())
}

/// A comma-separated list of numbers.
fn list<T: std::str::FromStr>(value: &str) -> Result<Vec<T>, String> {
    value
        .split(',')
        .map(|v| v.parse().map_err(|_| format!("{v}: not a number")))
        .collect()
}

fn build_dictionary(args: &[String]) -> Result<(), String> {
    let mut next = vec![32];
    let mut min_pair = 2;
    let mut date = 0;
    let mut files = Vec::new();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        let mut value = || rest.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--next" => next = list(value()?)?,
            "--min-pair" => min_pair = value()?.parse().map_err(|_| "--min-pair: not a number")?,
            "--date" => date = value()?.parse().map_err(|_| "--date: not a number")?,
            _ => files.push(PathBuf::from(arg)),
        }
    }
    let [old, out, train @ ..] = files.as_slice() else {
        return Err(USAGE.to_string());
    };
    let old_bytes = std::fs::read(old).map_err(|e| format!("{}: {e}", old.display()))?;
    let old = v202::read(&old_bytes)?;
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let texts = train
        .iter()
        .map(|f| read(f))
        .collect::<Result<Vec<_>, _>>()?;
    let lines: Vec<&str> = texts.iter().flat_map(|t| t.lines()).collect();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let counts = count::count(&old, &lines, threads);
    println!(
        "{} sentences, {} words, {} pairs",
        lines.len(),
        counts.tokens,
        counts.pairs.len()
    );
    for &next in &next {
        let options = dictionary::Options {
            next,
            min_pair,
            date,
        };
        let name = format!("next-{next}.dict");
        write_dictionary(&old, &counts, options, &out.join(name))?;
    }
    Ok(())
}

fn write_dictionary(
    old: &v202::Dictionary,
    counts: &count::Counts,
    options: dictionary::Options,
    file: &Path,
) -> Result<(), String> {
    let (built, summary) = dictionary::rebuild(old, counts, options);
    let bytes = v202::write(&built)?;
    std::fs::write(file, &bytes).map_err(|e| format!("{}: {e}", file.display()))?;
    println!(
        "{}: {} bytes; {} words seen; {} next words on {} words; scale {:?}",
        file.display(),
        bytes.len(),
        summary.words_seen,
        summary.bigrams,
        summary.words_with_bigrams,
        summary.scale
    );
    Ok(())
}
