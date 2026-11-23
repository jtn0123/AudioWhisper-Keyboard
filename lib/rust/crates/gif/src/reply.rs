//! How the GIF panel's requests are made and their answers read, as Kotlin's `Giphy` and
//! `GifImages` had it: how much of each body to read, how long to wait, which status counts, and
//! what a fetch of GIPHY's results comes to.

use wire::gif::{media, reply};

use crate::giphy::{self, Gif};
use crate::problem::Failure;

/// The only HTTP status whose body is read.
pub const OK_STATUS: i32 = 200;

/// A page of 30 results is about 100 KB; anything past this isn't a reply the panel can use.
pub const MAX_REPLY_BYTES: i32 = 2 * 1024 * 1024;

/// The longest preview the panel fetches.
pub const MAX_PREVIEW_BYTES: i32 = 4 * 1024 * 1024;

/// The longest GIF the panel sends.
pub const MAX_SEND_BYTES: i32 = 8 * 1024 * 1024;

/// How long GIPHY's API may take to connect, then to answer each read.
pub const API_TIMEOUT_MS: i32 = 10_000;

/// How long GIPHY's media servers may take to connect, then to answer each read.
pub const MEDIA_TIMEOUT_MS: i32 = 15_000;

/// How to make a request for a `GifMedia`: the longest body, the timeout and the status read,
/// as a `GifRequest` row.
pub fn request(media: i32) -> Option<[i32; 3]> {
    let max = match media {
        media::API => MAX_REPLY_BYTES,
        media::PREVIEW => MAX_PREVIEW_BYTES,
        media::SEND => MAX_SEND_BYTES,
        _ => return None,
    };
    let timeout = if media == media::API {
        API_TIMEOUT_MS
    } else {
        MEDIA_TIMEOUT_MS
    };
    Some([max, timeout, OK_STATUS])
}

/// What a fetch of GIPHY's results came to: a `GifReply` with its HTTP `status` and `body`.
/// A body that isn't the JSON expected is unreadable, as is one too long to read.
pub fn results(kind: &str, status: i32, body: &[u16]) -> Result<Result<Vec<Gif>, Failure>, String> {
    Ok(match kind {
        reply::OK if status == OK_STATUS => giphy::parse(body).map_err(|_| Failure::Unreadable),
        reply::OK | reply::STATUS => Err(Failure::Status(status)),
        reply::TOO_LARGE => Err(Failure::Unreadable),
        reply::UNREACHABLE => Err(Failure::Unreachable),
        _ => return Err(format!("no GIF reply {kind}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_media_has_its_limits() {
        assert_eq!(request(media::API), Some([2_097_152, 10_000, 200]));
        assert_eq!(request(media::PREVIEW), Some([4_194_304, 15_000, 200]));
        assert_eq!(request(media::SEND), Some([8_388_608, 15_000, 200]));
        assert_eq!(request(3), None);
    }

    #[test]
    fn a_reply_comes_to_gifs_or_a_failure() {
        let body = crate::utf16(r#"{"data":[]}"#);
        assert_eq!(results(reply::OK, 200, &body), Ok(Ok(Vec::new())));
        let bad = crate::utf16("not json");
        assert_eq!(results(reply::OK, 200, &bad), Ok(Err(Failure::Unreadable)));
        assert_eq!(
            results(reply::OK, 201, &body),
            Ok(Err(Failure::Status(201)))
        );
        assert_eq!(
            results(reply::STATUS, 429, &[]),
            Ok(Err(Failure::Status(429)))
        );
        assert_eq!(
            results(reply::TOO_LARGE, 200, &[]),
            Ok(Err(Failure::Unreadable))
        );
        assert_eq!(
            results(reply::UNREACHABLE, 0, &[]),
            Ok(Err(Failure::Unreachable))
        );
        assert!(results("other", 0, &[]).is_err());
    }
}
