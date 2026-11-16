//! `lexicon prepare OUT NAME=FILE [NAME=FILE ...]`: splits each source (one sentence a line) into
//! OUT/train-NAME.txt and OUT/test-NAME.txt (see `lexicon::corpus`).

use lexicon::corpus;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("prepare") => prepare(&args[1..]),
        _ => Err("usage: lexicon prepare OUT NAME=FILE [NAME=FILE ...]".to_string()),
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
