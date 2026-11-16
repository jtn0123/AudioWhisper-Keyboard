//! The `oracle` example's commands. `oracle gen COUNT SEED` writes COUNT seeded queries, one per
//! line; `oracle run` answers the queries on stdin with the C++ decoder and the dictionary the app ships,
//! one output line each. The lines are [`queries`]' format, which the emulator check reads and
//! writes too.

use std::io::{self, BufRead, BufWriter, Write};

use crate::queries::{self, Random};
use crate::{Dictionary, Proximity, Session, suggest};

/// Runs the command `args` (the program name first) names.
pub fn run(args: &[String]) -> io::Result<()> {
    let mut out = BufWriter::new(io::stdout().lock());
    let dictionary = Dictionary::shipped();
    match args.get(1).map(String::as_str) {
        Some("gen") => {
            let count: usize = args[2].parse().expect("COUNT");
            let mut random = Random::new(args[3].parse().expect("SEED"));
            let text = String::from_utf16(&dictionary.words()).expect("UTF-16");
            let words: Vec<Vec<char>> = text.lines().map(|it| it.chars().collect()).collect();
            for _ in 0..count {
                writeln!(
                    out,
                    "{}",
                    queries::query_line(&queries::query(&mut random, &words))
                )?;
            }
        }
        Some("run") => {
            let proximity = Proximity::qwerty();
            let session = Session::new("en_US", &dictionary);
            for line in io::stdin().lock().lines() {
                let query = queries::parse_query(&line?);
                let output = suggest(&dictionary, &proximity, &session, &query);
                writeln!(out, "{}", queries::output_line(&output))?;
            }
        }
        _ => eprintln!("usage: oracle gen COUNT SEED | oracle run < queries"),
    }
    out.flush()
}
