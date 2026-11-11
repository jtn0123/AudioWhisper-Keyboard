//! A panic aborts the process (see the release profile). On Android its message would go to
//! stderr, which nothing reads, so it is written to logcat first.
//!
//! Only where it happened is logged, not the message: some of the standard library's panic
//! messages quote the string involved (slicing text mid-character, for one), and that string
//! can be what the user typed. Logcat is readable over adb, so typed text never goes there.
//! Debug builds log the message too, since they only run on a developer's own phone.

/// Sends panic locations to logcat as fatal, under the tag "awnative".
#[cfg(target_os = "android")]
pub fn log_to_logcat() {
    use std::ffi::{CString, c_char, c_int};

    const ANDROID_LOG_FATAL: c_int = 7;

    #[link(name = "log")]
    unsafe extern "C" {
        fn __android_log_write(priority: c_int, tag: *const c_char, text: *const c_char) -> c_int;
    }

    std::panic::set_hook(Box::new(|info| {
        let message = if cfg!(debug_assertions) {
            info.to_string()
        } else {
            describe(info.location())
        };
        let text = CString::new(message.replace('\0', "")).unwrap_or_default();
        // SAFETY: both strings are NUL-terminated and outlive the call.
        unsafe { __android_log_write(ANDROID_LOG_FATAL, c"awnative".as_ptr(), text.as_ptr()) };
    }));
}

/// Off Android, as in the JVM tests on a computer, stderr is shown, so the default hook stays.
#[cfg(not(target_os = "android"))]
pub fn log_to_logcat() {}

/// The release build's panic line: where it happened, and nothing the user wrote.
#[cfg(any(target_os = "android", test))]
fn describe(location: Option<&std::panic::Location<'_>>) -> String {
    match location {
        Some(at) => format!("panicked at {}:{}:{}", at.file(), at.line(), at.column()),
        None => "panicked at an unknown location".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::describe;

    #[test]
    fn the_release_line_names_the_place_and_nothing_else() {
        let line = describe(Some(std::panic::Location::caller()));
        assert!(line.starts_with("panicked at "), "{line}");
        assert!(line.contains("panic.rs:"), "{line}");
        assert_eq!(describe(None), "panicked at an unknown location");
    }
}
