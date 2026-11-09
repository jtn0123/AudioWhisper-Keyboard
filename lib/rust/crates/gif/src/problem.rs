//! What a failed GIPHY call tells the user, as Kotlin's `GifSearch.problem` had it.

/// How a GIPHY call failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    /// GIPHY answered, but not with the JSON expected.
    Unreadable,
    /// GIPHY answered with this HTTP status.
    Status(i32),
    /// GIPHY couldn't be reached.
    Unreachable,
}

/// The message for `failure`.
pub fn problem(failure: Failure) -> &'static str {
    match failure {
        Failure::Unreadable => "GIPHY sent something unreadable. Try again later.",
        Failure::Status(401 | 403) => "GIPHY didn't accept the API key.",
        Failure::Status(429) => "GIPHY's hourly limit is used up. Try again later.",
        Failure::Status(_) => "GIPHY isn't answering right now.",
        Failure::Unreachable => "Couldn't reach GIPHY. Check your connection.",
    }
}
