//! Compiles LatinIME's C++ from `lib/latinime` (every source file there, so it is always the code
//! the app ships) with `cpp/fake_jni.cpp`, for this computer. Nothing is built for Android: the
//! oracle only runs in host tests.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        return;
    }
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let jni = manifest.join("../../../latinime/src/main/jni");
    let out = PathBuf::from(env::var("OUT_DIR").expect("set by cargo"));
    let include = out.join("include");
    fs::create_dir_all(&include).expect("make OUT_DIR/include");
    // Only jni.h: the NDK sysroot's other headers would shadow this computer's.
    fs::copy(find_jni_header(), include.join("jni.h")).expect("copy jni.h");

    let mut sources = cpp_files(&jni, false);
    sources.extend(cpp_files(&jni.join("src"), true));
    sources.sort();
    sources.push(manifest.join("cpp/fake_jni.cpp"));

    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .opt_level(2)
        .include(&include)
        .include(jni.join("src"))
        .files(&sources)
        .compile("latinime_oracle");

    println!("cargo::rerun-if-changed=cpp");
    println!("cargo::rerun-if-changed={}", jni.display());
    for name in [
        "ANDROID_NDK_HOME",
        "ANDROID_NDK_ROOT",
        "ANDROID_HOME",
        "ANDROID_SDK_ROOT",
    ] {
        println!("cargo::rerun-if-env-changed={name}");
    }
}

/// The `.cpp` files in `directory`, and its subdirectories' if `deep`.
fn cpp_files(directory: &Path, deep: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory).expect("read the LatinIME sources") {
        let path = entry.expect("read a directory entry").path();
        if path.is_dir() {
            if deep {
                files.extend(cpp_files(&path, true));
            }
        } else if path.extension().is_some_and(|it| it == "cpp") {
            files.push(path);
        }
    }
    files
}

/// The NDK's `jni.h`, the header the app's C++ is built against: from ANDROID_NDK_HOME or
/// ANDROID_NDK_ROOT, else the newest NDK in the SDK (ANDROID_HOME, ANDROID_SDK_ROOT, or the
/// usual place on macOS and Linux).
fn find_jni_header() -> PathBuf {
    let mut ndks: Vec<PathBuf> = ["ANDROID_NDK_HOME", "ANDROID_NDK_ROOT"]
        .iter()
        .filter_map(|name| env::var_os(name).map(PathBuf::from))
        .collect();
    let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let sdks = ["ANDROID_HOME", "ANDROID_SDK_ROOT"]
        .iter()
        .filter_map(|name| env::var_os(name).map(PathBuf::from))
        .chain([home.join("Library/Android/sdk"), home.join("Android/Sdk")]);
    for sdk in sdks {
        if let Ok(entries) = fs::read_dir(sdk.join("ndk")) {
            let mut versions: Vec<PathBuf> =
                entries.filter_map(|it| Some(it.ok()?.path())).collect();
            versions.sort();
            ndks.extend(versions.into_iter().rev());
        }
    }
    let prebuilt = ["darwin-x86_64", "linux-x86_64", "windows-x86_64"];
    ndks.iter()
        .flat_map(|ndk| {
            prebuilt.iter().map(move |host| {
                ndk.join("toolchains/llvm/prebuilt")
                    .join(host)
                    .join("sysroot/usr/include/jni.h")
            })
        })
        .find(|it| it.is_file())
        .expect("no Android NDK found: set ANDROID_NDK_HOME (the oracle needs the NDK's jni.h)")
}
