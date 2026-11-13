//! java.net.URI's validation for Snygg's fixed `flex:/` hierarchical scheme.
//! The URI is retained verbatim. Host lookup, resolution and file IO are never performed.
use crate::{text, value_spec::Failure};
fn invalid(reason: &str, index: usize, uri: &[u16]) -> Failure {
    Failure::Argument([text(&format!("{reason} at index {index}: ")), uri.to_vec()].concat())
}
fn hex(u: u16) -> bool {
    matches!(u,0x30..=0x39|0x41..=0x46|0x61..=0x66)
}
fn digit(u: u16) -> bool {
    matches!(u, 0x30..=0x39)
}
#[derive(Clone, Copy)]
enum Part {
    Path,
    Query,
    Authority,
    Server,
    User,
    Scope,
    Port,
}
impl Part {
    fn name(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Query => "query",
            Self::Authority | Self::Server => "authority",
            Self::User => "user info",
            Self::Scope => "scope id",
            Self::Port => "port number",
        }
    }
    fn accepts(self, u: u16) -> bool {
        if matches!(self, Self::Port) {
            return digit(u);
        }
        let alpha = matches!(u,0x41..=0x5a|0x61..=0x7a);
        if matches!(self, Self::Scope) {
            return alpha || digit(u) || matches!(u, 0x5f | 0x2e);
        }
        if u >= 128 {
            return u > 159 && !chars::is_whitespace(u);
        }
        if alpha || digit(u) || b"-_.!~*'()".iter().any(|&b| u == u16::from(b)) {
            return true;
        }
        let extra: &[u8] = match self {
            Self::Path => b":@&=+$,;/",
            Self::Query => b";/?:@&=+$,[]",
            Self::Authority => b"$,;:@&=+",
            Self::Server => b"$,;:@&=+[]",
            Self::User => b";:&=+$,",
            _ => b"",
        };
        extra.iter().any(|&b| u == u16::from(b))
    }
}
fn scan(
    uri: &[u16],
    start: usize,
    end: usize,
    part: Part,
    raw_percent: bool,
) -> Result<usize, Failure> {
    let mut at = start;
    while at < end {
        let u = uri[at];
        if part.accepts(u) || (raw_percent && u == 0x25) {
            at += 1;
            continue;
        }
        if u == 0x25 && !matches!(part, Part::Scope | Part::Port) {
            if at + 2 >= end || !hex(uri[at + 1]) || !hex(uri[at + 2]) {
                return Err(invalid("Malformed escape pair", at, uri));
            }
            at += 3;
            continue;
        }
        break;
    }
    Ok(at)
}
fn check(uri: &[u16], start: usize, end: usize, part: Part) -> Result<(), Failure> {
    let at = scan(uri, start, end, part, false)?;
    if at < end {
        Err(invalid(
            &format!("Illegal character in {}", part.name()),
            at,
            uri,
        ))
    } else {
        Ok(())
    }
}
pub fn validate(uri: &[u16]) -> Result<(), Failure> {
    if !uri.starts_with(&text("flex:/")) {
        return Err(Failure::argument("Expected flex URI"));
    }
    let n = uri.len();
    let mut start = 5;
    if uri.get(5..7) == Some(&[0x2f, 0x2f]) {
        start = 7;
        let end = uri[start..]
            .iter()
            .position(|&u| matches!(u, 0x2f | 0x3f | 0x23))
            .map_or(n, |i| start + i);
        if end > start {
            authority(uri, start, end)?;
            start = end
        } else if end == n {
            return Err(invalid("Expected authority", start, uri));
        }
    }
    let end = uri[start..]
        .iter()
        .position(|&u| matches!(u, 0x3f | 0x23))
        .map_or(n, |i| start + i);
    check(uri, start, end, Part::Path)?;
    start = end;
    if uri.get(start) == Some(&0x3f) {
        start += 1;
        let end = uri[start..]
            .iter()
            .position(|&u| u == 0x23)
            .map_or(n, |i| start + i);
        check(uri, start, end, Part::Query)?;
        start = end;
    }
    if uri.get(start) == Some(&0x23) {
        start += 1;
        let at = scan(uri, start, n, Part::Query, false)?;
        if at < n {
            return Err(invalid("Illegal character in fragment", at, uri));
        }
    }
    Ok(())
}
fn authority(uri: &[u16], start: usize, end: usize) -> Result<(), Failure> {
    let raw = uri.get(start) != Some(&0x5d);
    let server = scan(uri, start, end, Part::Server, raw)? == end;
    let reg = scan(uri, start, end, Part::Authority, false)?;
    if reg == end {
        return Ok(());
    }
    if !server {
        return Err(invalid("Illegal character in authority", reg, uri));
    }
    let mut at = start;
    if let Some(i) = uri[start..end].iter().position(|&u| u == 0x40) {
        check(uri, start, start + i, Part::User)?;
        at = start + i + 1
    }
    if uri.get(at) != Some(&0x5b) {
        let mut last = None;
        loop {
            let begin = at;
            while at < end && (digit(uri[at]) || matches!(uri[at],0x41..=0x5a|0x61..=0x7a)) {
                at += 1;
            }
            if at == begin {
                break;
            }
            last = Some(begin);
            while at < end && (digit(uri[at]) || matches!(uri[at],0x41..=0x5a|0x61..=0x7a|0x2d)) {
                at += 1;
            }
            if uri.get(at.wrapping_sub(1)) == Some(&0x2d) {
                return Err(invalid("Illegal character in hostname", at - 1, uri));
            }
            if uri.get(at) != Some(&0x2e) {
                break;
            }
            at += 1;
        }
        if at < end && uri[at] != 0x3a {
            return Err(invalid("Illegal character in hostname", at, uri));
        }
        let Some(last) = last else {
            return Err(invalid("Expected hostname", at, uri));
        };
        if last > start && !matches!(uri[last],0x41..=0x5a|0x61..=0x7a) {
            return Err(invalid("Illegal character in hostname", last, uri));
        }
        return Err(invalid("Illegal character in port number", at + 1, uri));
    }
    at += 1;
    let close = uri[at..end]
        .iter()
        .position(|&u| u == 0x5d)
        .map_or(end, |i| at + i);
    if close == at || close == end {
        return Err(invalid(
            "Expected closing bracket for IPv6 address",
            close,
            uri,
        ));
    }
    let scope = uri[at..close]
        .iter()
        .position(|&u| u == 0x25)
        .map_or(close, |i| at + i);
    ipv6(uri, at, scope)?;
    if scope < close {
        if scope + 1 == close {
            return Err(Failure::Argument(
                [text("scope id expected: "), uri.to_vec()].concat(),
            ));
        }
        check(uri, scope + 1, close, Part::Scope)?;
    }
    at = close + 1;
    if uri.get(at) == Some(&0x3a) {
        at += 1;
        check(uri, at, end, Part::Port)?;
        if at < end && chars::text::to_int_or_null(&uri[at..end]).is_none() {
            return Err(invalid("Malformed port number", at, uri));
        }
        at = end;
    }
    if at < end {
        return Err(invalid("Expected port number", at, uri));
    }
    Ok(())
}
fn hex_seq(
    uri: &[u16],
    start: usize,
    end: usize,
    bytes: &mut usize,
) -> Result<Option<usize>, Failure> {
    let mut at = start;
    let mut next = at;
    while next < end && hex(uri[next]) {
        next += 1
    }
    if next == at || uri.get(next) == Some(&0x2e) {
        return Ok(None);
    }
    if next - at > 4 {
        return Err(invalid("IPv6 hexadecimal digit sequence too long", at, uri));
    }
    *bytes += 2;
    at = next;
    while at < end && uri[at] == 0x3a && uri.get(at + 1) != Some(&0x3a) {
        at += 1;
        next = at;
        while next < end && hex(uri[next]) {
            next += 1
        }
        if next == at {
            return Err(invalid("Expected digits for an IPv6 address", at, uri));
        }
        if uri.get(next) == Some(&0x2e) {
            at -= 1;
            break;
        }
        if next - at > 4 {
            return Err(invalid("IPv6 hexadecimal digit sequence too long", at, uri));
        }
        *bytes += 2;
        at = next;
    }
    Ok(Some(at))
}
fn ipv4(uri: &[u16], start: usize, end: usize, expected: &str) -> Result<usize, Failure> {
    if start == end || !uri[start..end].iter().all(|&u| digit(u) || u == 0x2e) {
        return Err(invalid(&format!("Expected {expected}"), start, uri));
    }
    let mut at = start;
    for group in 0..4 {
        let begin = at;
        while at < end && digit(uri[at]) {
            at += 1
        }
        if at == begin || chars::text::to_int_or_null(&uri[begin..at]).is_none_or(|n| n > 255) {
            return Err(invalid("Malformed IPv4 address", begin, uri));
        }
        if group < 3 {
            if uri.get(at) != Some(&0x2e) {
                return Err(invalid("Malformed IPv4 address", at, uri));
            }
            at += 1
        }
    }
    if at < end {
        return Err(invalid("Malformed IPv4 address", at, uri));
    }
    Ok(at)
}
fn ipv6(uri: &[u16], start: usize, end: usize) -> Result<(), Failure> {
    let mut bytes = 0;
    let mut compressed = false;
    let mut at = hex_seq(uri, start, end, &mut bytes)?.unwrap_or(start);
    if uri.get(at..at + 2) == Some(&[0x3a, 0x3a]) {
        compressed = true;
        at += 2;
        if at < end {
            if let Some(next) = hex_seq(uri, at, end, &mut bytes)? {
                at = next;
                if uri.get(at) == Some(&0x3a) {
                    at = ipv4(uri, at + 1, end, "hex digits or IPv4 address")?;
                    bytes += 4
                }
            } else {
                at = ipv4(uri, at, end, "hex digits or IPv4 address")?;
                bytes += 4
            }
        }
    } else if at > start && uri.get(at) == Some(&0x3a) {
        at = ipv4(uri, at + 1, end, "IPv4 address")?;
        bytes += 4
    }
    let error = if at < end {
        Some("Malformed IPv6 address")
    } else if bytes > 16 {
        Some("IPv6 address too long")
    } else if !compressed && bytes < 16 {
        Some("IPv6 address too short")
    } else if compressed && bytes == 16 {
        Some("Malformed IPv6 address")
    } else {
        None
    };
    if let Some(reason) = error {
        Err(invalid(reason, start, uri))
    } else {
        Ok(())
    }
}
