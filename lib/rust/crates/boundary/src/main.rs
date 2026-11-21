//! `cargo run -p boundary -- DIR`: writes the Kotlin side of the generated boundary under DIR, a
//! Kotlin source root. Gradle's `generateNativeBoundary` task runs it, into lib/rust's build
//! directory, before Kotlin compiles; awnative's build.rs writes the Rust side.

use std::path::Path;
use std::process::ExitCode;
use std::{env, fs, io};

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let (Some(root), None) = (args.next(), args.next()) else {
        eprintln!("usage: boundary DIR");
        return ExitCode::FAILURE;
    };
    for file in boundary::kotlin_files(boundary::CLASSES) {
        let path = Path::new(&root).join(&file.path);
        if let Err(error) = write(&path, &file.text) {
            eprintln!("boundary: cannot write {}: {error}", path.display());
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

fn write(path: &Path, text: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)
}
