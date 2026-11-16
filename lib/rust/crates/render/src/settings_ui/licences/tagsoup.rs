//! TagSoup 1.2's HTMLScanner and Parser, as `Html.fromHtml` drives them on Android, reduced to
//! the events that reach its text: element starts and ends (by local name) and characters.
//! Attributes, comments, declarations and processing instructions are scanned and dropped.
//! Text stays in UTF-16 code units, as Java's `char` buffers hold it; `scanner.rs` feeds it.
use super::schema;
use std::{collections::HashMap, sync::OnceLock};

const F_RESTART: u32 = 1;
const F_CDATA: u32 = 2;
const F_NOFORCE: u32 = 4;

/// What `Html`'s SAX content handler hears.
pub(super) trait Handler {
    fn start(&mut self, local: &str);
    fn end(&mut self, local: &str);
    fn characters(&mut self, text: &[u16]);
}
#[derive(Clone, Debug)]
pub(super) struct Type {
    name: String,
    model: u32,
    member: u32,
    flags: u32,
    parent: Option<String>,
}
pub(super) struct Schema {
    pub(super) types: HashMap<String, Type>,
    entities: HashMap<&'static str, u32>,
}
pub(super) fn schema() -> &'static Schema {
    static SCHEMA: OnceLock<Schema> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let hex = |s: &str| u32::from_str_radix(s, 16).expect("schema hex");
        let types = schema::ELEMENTS
            .split_whitespace()
            .map(|entry| {
                let f: Vec<&str> = entry.split(',').collect();
                let t = Type {
                    name: f[0].into(),
                    model: hex(f[1]),
                    member: hex(f[2]),
                    flags: hex(f[3]),
                    parent: (f[4] != "-").then(|| f[4].to_owned()),
                };
                (f[0].to_lowercase(), t)
            })
            .collect();
        let entities = schema::ENTITIES
            .split_whitespace()
            .map(|entry| {
                let (name, value) = entry.split_once(',').expect("entity entry");
                (name, hex(value))
            })
            .collect();
        Schema { types, entities }
    })
}
#[derive(Clone, Debug)]
pub(super) struct Element {
    pub(super) t: Type,
    pub(super) preclosed: bool,
}
impl Element {
    fn can_contain(&self, other: &Type) -> bool {
        self.t.model & other.member != 0
    }
}
/// Java's `Character.isWhitespace` for one UTF-16 unit.
pub(super) fn java_whitespace(c: u16) -> bool {
    matches!(c, 0x09..=0x0D | 0x1C..=0x20)
        || (c > 0x7F
            && !matches!(c, 0xA0 | 0x2007 | 0x202F)
            && char::from_u32(c.into()).is_some_and(char::is_whitespace)
            && c != 0x85)
}
/// `Character.toLowerCase(char)`: a unit whose lowercase is one unit maps to it.
fn java_lower(c: u16) -> u16 {
    let Some(ch) = unit_char(c) else { return c };
    let mut lower = ch.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(l), None) if (l as u32) <= 0xFFFF => l as u16,
        _ => c,
    }
}
fn unit_char(c: u16) -> Option<char> {
    char::from_u32(c.into())
}
pub(super) fn is_letter(c: u16) -> bool {
    unit_char(c).is_some_and(|c| c.is_alphabetic() && !c.is_numeric())
}
pub(super) fn is_digit(c: u16) -> bool {
    unit_char(c).is_some_and(|c| c.is_ascii_digit() || (!c.is_ascii() && c.is_numeric()))
}
/// TagSoup's `makeName`: letters and `_` kept, digits `-` `.` kept (`_` first at a start), one
/// colon kept, everything else dropped; an empty or colon-ended name gains `_`.
fn make_name(buf: &[u16]) -> String {
    let mut out: Vec<u16> = vec![];
    let (mut seen_colon, mut start) = (false, true);
    for &c in buf {
        if is_letter(c) || c == u16::from(b'_') {
            start = false;
            out.push(c);
        } else if is_digit(c) || c == u16::from(b'-') || c == u16::from(b'.') {
            if start {
                out.push(u16::from(b'_'));
            }
            start = false;
            out.push(c);
        } else if c == u16::from(b':') && !seen_colon {
            seen_colon = true;
            if start {
                out.push(u16::from(b'_'));
            }
            start = true;
            out.push(c);
        }
    }
    if out.last().is_none_or(|&c| c == u16::from(b':')) {
        out.push(u16::from(b'_'));
    }
    String::from_utf16_lossy(&out)
}
fn local(name: &str) -> &str {
    name.split_once(':').map_or(name, |(_, l)| l)
}

pub(super) struct Parser<'h, H: Handler> {
    pub(super) h: &'h mut H,
    pub(super) bogons: HashMap<String, Type>,
    pub(super) stack: Vec<Element>,
    pub(super) saved: Vec<Element>,
    pub(super) new_element: Option<Element>,
    pub(super) virgin: bool,
    pub(super) cdata: bool,
    pub(super) entity: u32,
}
impl<H: Handler> Parser<'_, H> {
    fn lookup(&self, name: &str) -> Option<Type> {
        let key = name.to_lowercase();
        schema()
            .types
            .get(&key)
            .or_else(|| self.bogons.get(&key))
            .cloned()
    }
    fn pcdata_type() -> Type {
        schema().types["<pcdata>"].clone()
    }
    pub(super) fn top(&self) -> &Element {
        self.stack.last().expect("the root never pops")
    }
    fn push(&mut self, e: Element) {
        self.h.start(local(&e.t.name));
        let cdata = e.t.flags & F_CDATA != 0;
        self.stack.push(e);
        self.virgin = false;
        if cdata {
            self.cdata = true;
        }
    }
    fn pop(&mut self) {
        if let Some(e) = self.stack.pop() {
            self.h.end(local(&e.t.name));
        }
    }
    fn restartably_pop(&mut self) {
        let Some(e) = self.stack.last().cloned() else {
            return;
        };
        self.pop();
        if e.t.flags & F_RESTART != 0 {
            self.saved.push(e);
        }
    }
    fn restart(&mut self, next: Option<&Type>) {
        while let Some(s) = self.saved.last()
            && self.top().can_contain(&s.t)
            && next.is_none_or(|n| s.t.model & n.member != 0)
        {
            let s = self.saved.pop().expect("checked");
            self.push(s);
        }
    }
    pub(super) fn rectify(&mut self, e: Type) {
        let mut chain = vec![e];
        let at = loop {
            if let Some(at) = self.stack.iter().rposition(|s| s.can_contain(&chain[0])) {
                break Some(at);
            }
            match chain[0].parent.as_deref().and_then(|p| self.lookup(p)) {
                Some(parent) => chain.insert(0, parent),
                None => break None,
            }
        };
        let Some(at) = at else { return };
        while self.stack.len() - 1 != at && self.stack.len() > 2 {
            self.restartably_pop();
        }
        for i in 0..chain.len() {
            if chain[i].name != "<pcdata>" {
                self.push(Element {
                    t: chain[i].clone(),
                    preclosed: false,
                });
            }
            self.restart(chain.get(i + 1));
        }
        self.new_element = None;
    }
    pub(super) fn gi(&mut self, buf: &[u16]) {
        if self.new_element.is_some() {
            return;
        }
        let name = make_name(buf);
        let t = self.lookup(&name).unwrap_or_else(|| {
            // Root bogons (TagSoup's default): any model, a member of every model.
            let t = Type {
                name: name.clone(),
                model: u32::MAX,
                member: u32::MAX,
                flags: 0,
                parent: None,
            };
            self.bogons.insert(name.to_lowercase(), t.clone());
            t
        });
        self.new_element = Some(Element {
            t,
            preclosed: false,
        });
    }
    pub(super) fn stagc(&mut self) {
        let Some(e) = self.new_element.clone() else {
            return;
        };
        self.rectify(e.t);
        if self.top().t.model == 0 {
            self.etag_basic(&[]);
        }
    }
    pub(super) fn etag(&mut self, buf: &[u16]) {
        if self.cdata_mode_etag(buf) {
            return;
        }
        self.etag_basic(buf);
    }
    fn cdata_mode_etag(&mut self, buf: &[u16]) -> bool {
        let top = &self.top().t;
        if top.flags & F_CDATA == 0 {
            return false;
        }
        let name: Vec<u16> = top.name.encode_utf16().collect();
        let real = buf.len() == name.len()
            && buf
                .iter()
                .zip(&name)
                .all(|(&a, &b)| java_lower(a) == java_lower(b));
        if real {
            return false;
        }
        self.h.characters(&[u16::from(b'<'), u16::from(b'/')]);
        self.h.characters(buf);
        self.h.characters(&[u16::from(b'>')]);
        self.cdata = true;
        true
    }
    pub(super) fn etag_basic(&mut self, buf: &[u16]) {
        self.new_element = None;
        let name = if buf.is_empty() {
            self.top().t.name.clone()
        } else {
            match self.lookup(&make_name(buf)) {
                Some(t) => t.name,
                None => return,
            }
        };
        let mut noforce = false;
        let mut found = None;
        for (i, e) in self.stack.iter().enumerate().rev() {
            if e.t.name == name {
                found = Some(i);
                break;
            }
            if e.t.flags & F_NOFORCE != 0 {
                noforce = true;
            }
        }
        let Some(at) = found else { return };
        if at < 2 {
            return;
        }
        if noforce {
            self.stack[at].preclosed = true;
        } else {
            while self.stack.len() - 1 > at {
                self.restartably_pop();
            }
            self.pop();
        }
        while self.top().preclosed {
            self.pop();
        }
        self.restart(None);
    }
    pub(super) fn pcdata(&mut self, buf: &[u16]) {
        if buf.is_empty() {
            return;
        }
        let white = buf.iter().all(|&c| java_whitespace(c));
        let pcdata = Self::pcdata_type();
        if white && self.top().t.model & pcdata.member == 0 {
            return;
        }
        self.rectify(pcdata);
        self.h.characters(buf);
    }
    pub(super) fn entity(&mut self, buf: &[u16]) {
        self.entity = lookup_entity(buf);
    }
    pub(super) fn eof(&mut self) {
        if self.virgin {
            self.rectify(Self::pcdata_type());
        }
        while self.stack.len() > 1 {
            self.pop();
        }
    }
}
/// TagSoup's `lookupEntity`: numeric references in decimal or hex (0 when they do not parse
/// as an int), named ones from the schema (0 when unknown).
pub(super) fn lookup_entity(buf: &[u16]) -> u32 {
    let Some(&first) = buf.first() else { return 0 };
    let text = String::from_utf16_lossy(buf);
    if first == u16::from(b'#') {
        if buf.len() > 1 && (buf[1] == u16::from(b'x') || buf[1] == u16::from(b'X')) {
            return i32::from_str_radix(&text[2..], 16).map_or(0, |v| v as u32);
        }
        return text[1..].parse::<i32>().map_or(0, |v| v as u32);
    }
    schema().entities.get(text.as_str()).copied().unwrap_or(0)
}
