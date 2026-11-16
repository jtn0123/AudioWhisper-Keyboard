//! `AnnotatedString.fromHtml`'s text: Compose prefixes `<ContentHandlerReplacementTag />` and
//! calls `HtmlCompat.fromHtml(FROM_HTML_MODE_COMPACT)`, whose `Html` content handler builds the
//! text from TagSoup's events. Only the text is kept; the spans it would attach are dropped.
use super::{scanner, tagsoup::Handler};

/// The text `Html.HtmlToSpannedConverter` appends for TagSoup's events in compact mode, where
/// every paragraph, list, item, div, quote and heading margin is one line break.
#[derive(Default)]
struct Text {
    out: Vec<u16>,
    /// The open block elements' `Newline` marks.
    marks: usize,
}
const NL: u16 = b'\n' as u16;
const SP: u16 = b' ' as u16;
fn block(tag: &str) -> bool {
    let t = tag.to_ascii_lowercase();
    matches!(t.as_str(), "p" | "ul" | "li" | "div" | "blockquote")
        || (t.len() == 2 && t.starts_with('h') && (b'1'..=b'6').contains(&t.as_bytes()[1]))
}
impl Text {
    /// `appendNewlines`: at least `min` trailing line breaks, none at the very start.
    fn newlines(&mut self, min: usize) {
        if self.out.is_empty() {
            return;
        }
        let existing = self.out.iter().rev().take_while(|&&c| c == NL).count();
        for _ in existing..min {
            self.out.push(NL);
        }
    }
}
impl Handler for Text {
    fn start(&mut self, local: &str) {
        if block(local) {
            self.newlines(1);
            self.marks += 1;
        } else if local.eq_ignore_ascii_case("img") {
            self.out.push(0xFFFC);
        }
    }
    fn end(&mut self, local: &str) {
        if local.eq_ignore_ascii_case("br") {
            self.out.push(NL);
        } else if block(local) && self.marks > 0 {
            self.marks -= 1;
            self.newlines(1);
        }
    }
    /// Spaces and line breaks collapse into one space, dropped after a space or line break.
    fn characters(&mut self, text: &[u16]) {
        let start = self.out.len();
        for &c in text {
            if c == SP || c == NL {
                let pred = if self.out.len() > start {
                    self.out[self.out.len() - 1]
                } else {
                    self.out.last().copied().unwrap_or(NL)
                };
                if pred != SP && pred != NL {
                    self.out.push(SP);
                }
            } else {
                self.out.push(c);
            }
        }
    }
}
/// The text `AnnotatedString.fromHtml(html)` shows.
pub fn from_html(html: &str) -> String {
    let input: Vec<u16> = "<ContentHandlerReplacementTag />"
        .encode_utf16()
        .chain(html.encode_utf16())
        .collect();
    let mut text = Text::default();
    scanner::parse(&input, &mut text);
    String::from_utf16_lossy(&text.out)
}
