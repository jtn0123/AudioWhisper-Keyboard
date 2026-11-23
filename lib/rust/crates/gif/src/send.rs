//! Sending one GIF into the app being typed in, as Kotlin's `GifSender` had it: as an image if
//! the app takes GIFs, downloaded once into a part file and renamed only when whole, else its
//! page link typed. Kotlin does each step and answers with what came of it ([`next`]).

use wire::gif::send::*;

/// Sent GIFs are kept this long, so the receiving app can still read one it hasn't sent yet.
pub const KEEP_MS: i64 = 24 * 60 * 60 * 1000;

/// What a download in progress is written to, before it is renamed.
pub const PART: &str = ".part";

/// Longer than a download can take: two 15-second timeouts.
pub const PART_STALE_MS: i64 = 60_000;

/// What to do after `step`, which worked or not (`ok`); `linkable` says the GIF has a page link.
pub fn next(step: &str, ok: bool, linkable: bool) -> Result<&'static [&'static str], String> {
    let link: &'static [&'static str] = if linkable { &[TYPE_LINK] } else { &[FAILED] };
    Ok(match (step, ok) {
        (START, true) => &[PRUNE],
        (START | COMMITTED, false) => link,
        (CHECKED, true) => &[TOUCH, COMMIT],
        (CHECKED, false) => &[DOWNLOAD],
        (SAVED, true) => &[RENAME],
        (SAVED | RENAMED, false) => &[DELETE_PART, FAILED],
        (RENAMED, true) => &[COMMIT],
        (COMMITTED, true) => &[SENT],
        (TYPED, true) => &[LINKED],
        (TYPED, false) => &[FAILED],
        _ => return Err(format!("no GIF send step {step}")),
    })
}

/// The file a GIF with `id` is sent from: its letters and digits, or "gif", then ".gif".
pub fn file_name(id: &[u16]) -> Vec<u16> {
    let mut name: Vec<u16> = id
        .iter()
        .copied()
        .filter(|&unit| chars::is_letter_or_digit(unit))
        .collect();
    if name.is_empty() {
        name = crate::utf16("gif");
    }
    name.extend(".gif".encode_utf16());
    name
}

/// Whether the file `name`, last changed `age` ago, is pruned: a sent GIF after a day, a part
/// file once no download can still be writing it.
pub fn stale(name: &[u16], age: i64) -> bool {
    let part: Vec<u16> = crate::utf16(PART);
    if name.ends_with(&part) {
        age > PART_STALE_MS
    } else {
        age > KEEP_MS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utf16;

    #[test]
    fn a_gif_goes_as_an_image_or_its_link() {
        assert_eq!(next(START, true, true), Ok(&[PRUNE][..]));
        assert_eq!(next(START, false, true), Ok(&[TYPE_LINK][..]));
        assert_eq!(next(START, false, false), Ok(&[FAILED][..]));
        assert_eq!(next(CHECKED, true, true), Ok(&[TOUCH, COMMIT][..]));
        assert_eq!(next(CHECKED, false, true), Ok(&[DOWNLOAD][..]));
        assert_eq!(next(SAVED, true, false), Ok(&[RENAME][..]));
        assert_eq!(next(SAVED, false, true), Ok(&[DELETE_PART, FAILED][..]));
        assert_eq!(next(RENAMED, true, true), Ok(&[COMMIT][..]));
        assert_eq!(next(RENAMED, false, true), Ok(&[DELETE_PART, FAILED][..]));
        assert_eq!(next(COMMITTED, true, false), Ok(&[SENT][..]));
        assert_eq!(next(COMMITTED, false, true), Ok(&[TYPE_LINK][..]));
        assert_eq!(next(COMMITTED, false, false), Ok(&[FAILED][..]));
        assert_eq!(next(TYPED, true, true), Ok(&[LINKED][..]));
        assert_eq!(next(TYPED, false, true), Ok(&[FAILED][..]));
        assert!(next("other", true, true).is_err());
    }

    #[test]
    fn names_keep_letters_and_digits() {
        assert_eq!(file_name(&utf16("a/b-c9")), utf16("abc9.gif"));
        assert_eq!(file_name(&utf16("../")), utf16("gif.gif"));
        assert_eq!(file_name(&utf16("é中")), utf16("é中.gif"));
        assert_eq!(file_name(&[0xD83D, 0xDE00]), utf16("gif.gif"));
    }

    #[test]
    fn old_gifs_and_stale_parts_are_pruned() {
        assert!(!stale(&utf16("a.gif"), KEEP_MS));
        assert!(stale(&utf16("a.gif"), KEEP_MS + 1));
        assert!(!stale(&utf16("a.gif.part"), PART_STALE_MS));
        assert!(stale(&utf16("a.gif.part"), PART_STALE_MS + 1));
        assert!(!stale(&utf16("a.gif"), -1));
    }
}
