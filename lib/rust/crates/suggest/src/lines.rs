//! A saved file's lines, as Kotlin's `File.readLines` gives them to the stores that read one.

const LINE_FEED: u16 = b'\n' as u16;
const CARRIAGE_RETURN: u16 = b'\r' as u16;

/// The lines of `text`, split as `BufferedReader.readLine` splits them: at "\n", "\r" or "\r\n",
/// with no empty line after a final line break.
pub(crate) fn lines(text: &[u16]) -> Vec<&[u16]> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < text.len() {
        if text[i] == LINE_FEED || text[i] == CARRIAGE_RETURN {
            lines.push(&text[start..i]);
            if text[i] == CARRIAGE_RETURN && text.get(i + 1) == Some(&LINE_FEED) {
                i += 1;
            }
            start = i + 1;
        }
        i += 1;
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(text: &str) -> Vec<String> {
        let units: Vec<u16> = text.encode_utf16().collect();
        lines(&units)
            .into_iter()
            .map(String::from_utf16_lossy)
            .collect()
    }

    #[test]
    fn splits_as_read_lines_does() {
        assert_eq!(split(""), Vec::<String>::new());
        assert_eq!(split("a"), ["a"]);
        assert_eq!(split("a\n"), ["a"]);
        assert_eq!(split("a\nb"), ["a", "b"]);
        assert_eq!(split("a\r\nb\rc\n\nd"), ["a", "b", "c", "", "d"]);
        assert_eq!(split("\n\r\n\r"), ["", "", ""]);
        assert_eq!(split("a\r"), ["a"]);
        assert_eq!(split("a\n\r"), ["a", ""]);
    }
}
