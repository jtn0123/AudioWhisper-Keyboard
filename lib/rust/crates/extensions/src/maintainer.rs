//! The former ExtensionMaintainer regex and substring plan, in UTF-16. Regex character
//! classes use the generated JVM/Android character tables, including their differences.
use crate::Text;

fn name_code(cp: u32) -> bool {
    chars::code_point::is_letter(cp)
        || if cfg!(target_os = "android") {
            chars::code_point::is_digit(cp)
        } else {
            (48..=57).contains(&cp)
        }
        || [46, 95, 45].contains(&cp)
}
fn space(cp: u32) -> bool {
    u16::try_from(cp).is_ok_and(chars::is_regex_space)
}

fn matches(input: &[u16]) -> bool {
    let cps = chars::text::code_points(input)
        .map(|(_, cp)| cp)
        .collect::<Vec<_>>();
    let mut at = 0;
    while cps.get(at).is_some_and(|&cp| space(cp)) {
        at += 1;
    }
    if !cps.get(at).is_some_and(|&cp| name_code(cp)) {
        return false;
    }
    at += 1;
    while cps.get(at).is_some_and(|&cp| name_code(cp) || space(cp)) {
        at += 1;
    }
    if cps.get(at) == Some(&60) {
        let begin = at + 1;
        at = begin;
        while cps.get(at).is_some_and(|&cp| cp != 60 && cp != 62) {
            at += 1;
        }
        if at == begin || cps.get(at) != Some(&62) {
            return false;
        }
        at += 1;
    }
    while cps.get(at).is_some_and(|&cp| space(cp)) {
        at += 1;
    }
    if cps.get(at) == Some(&40) {
        let begin = at + 1;
        at = begin;
        while cps.get(at).is_some_and(|&cp| cp != 40 && cp != 41) {
            at += 1;
        }
        if at == begin || cps.get(at) != Some(&41) {
            return false;
        }
        at += 1;
    }
    while cps.get(at).is_some_and(|&cp| space(cp)) {
        at += 1;
    }
    at == cps.len()
}

/// Offsets are a transfer plan, so Kotlin's substring supplies the platform's exact exception
/// class/message for the historical regex's unusual accepted but reversed delimiter ranges.
pub fn parse(input: &[u16]) -> Option<[i32; 6]> {
    if !matches(input) {
        return None;
    }
    let size = input.len();
    let start = |c| input.iter().position(|&u| u == c).map_or(size, |i| i + 1);
    let end = |c| input.iter().position(|&u| u == c).unwrap_or(size);
    let (email_start, email_end, url_start, url_end) = (start(60), end(62), start(40), end(41));
    let name_end = if email_start == size && url_start == size {
        size
    } else {
        email_start.min(url_start) - 1
    };
    let trimmed = |mut lo: usize, mut hi: usize, nullable| {
        if lo <= hi {
            while lo < hi && chars::is_whitespace(input[lo]) {
                lo += 1;
            }
            while lo < hi && chars::is_whitespace(input[hi - 1]) {
                hi -= 1;
            }
            if nullable && lo == hi {
                return [-1, -1];
            }
        }
        [lo as i32, hi as i32]
    };
    let mut ranges = [0; 6];
    ranges[..2].copy_from_slice(&trimmed(0, name_end, false));
    ranges[2..4].copy_from_slice(&trimmed(email_start, email_end, true));
    ranges[4..].copy_from_slice(&trimmed(url_start, url_end, true));
    Some(ranges)
}

pub fn format(name: &[u16], email: &[u16], url: &[u16]) -> Text {
    let mut out = name.to_vec();
    for (value, begin, end) in [(email, " <", ">"), (url, " (", ")")] {
        if !value.iter().all(|&u| chars::is_whitespace(u)) {
            out.extend(begin.encode_utf16());
            out.extend(value);
            out.extend(end.encode_utf16());
        }
    }
    out
}
