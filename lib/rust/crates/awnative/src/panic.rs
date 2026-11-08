//! A panic aborts the process (see the release profile). On Android its message would go to
//! stderr, which nothing reads, so it is written to logcat first.

/// Sends panic messages to logcat as fatal, under the tag "awnative".
#[cfg(target_os = "android")]
pub fn log_to_logcat() {
    use std::ffi::{CString, c_char, c_int};

    const ANDROID_LOG_FATAL: c_int = 7;

    #[link(name = "log")]
    unsafe extern "C" {
        fn __android_log_write(priority: c_int, tag: *const c_char, text: *const c_char) -> c_int;
    }

    std::panic::set_hook(Box::new(|info| {
        let text = CString::new(info.to_string().replace('\0', "")).unwrap_or_default();
        // SAFETY: both strings are NUL-terminated and outlive the call.
        unsafe { __android_log_write(ANDROID_LOG_FATAL, c"awnative".as_ptr(), text.as_ptr()) };
    }));
}

/// Off Android, as in the JVM tests on a computer, stderr is shown, so the default hook stays.
#[cfg(not(target_os = "android"))]
pub fn log_to_logcat() {}
