//! `cargo run -p boundary -- RUST KOTLIN`: writes the Kotlin side of the generated boundary under
//! RUST and KOTLIN, the generated source roots of lib/rust and lib/kotlin. Gradle's
//! `generateNativeBoundary` task runs it, into lib/rust's build directory, before Kotlin compiles;
//! awnative's build.rs writes the Rust side.

use std::path::Path;
use std::process::ExitCode;
use std::{env, fs, io};

use boundary::Module;

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let (Some(rust), Some(kotlin), None) = (args.next(), args.next(), args.next()) else {
        eprintln!("usage: boundary RUST KOTLIN");
        return ExitCode::FAILURE;
    };
    let classes = boundary::kotlin_files(boundary::CLASSES)
        .into_iter()
        .map(|file| (Module::Rust, file));
    for (module, file) in classes.chain(boundary::wire_files(boundary::WIRE)) {
        let root = match module {
            Module::Rust => &rust,
            Module::Kotlin => &kotlin,
        };
        let path = Path::new(root).join(&file.path);
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
