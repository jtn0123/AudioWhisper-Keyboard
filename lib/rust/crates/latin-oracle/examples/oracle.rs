//! `oracle gen COUNT SEED | oracle run`: see `latin_oracle::cli`. The oracle is LatinIME's C++
//! built for this computer, so on Android there is nothing to run.

#[cfg(not(target_os = "android"))]
fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    latin_oracle::cli::run(&args)
}

#[cfg(target_os = "android")]
fn main() {
    eprintln!("the oracle runs on the computer that builds the app, not on Android");
}
