//! Writes the JNI entry points crates/boundary generates from its declarations into cargo's
//! OUT_DIR, where src/boundary.rs includes them. Cargo runs this again whenever the declarations
//! change, since they are a build dependency.

use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    let stubs = boundary::rust_file(boundary::CLASSES);
    fs::write(out.join("boundary.rs"), stubs).expect("OUT_DIR is writable");
    println!("cargo::rerun-if-changed=build.rs");
}
