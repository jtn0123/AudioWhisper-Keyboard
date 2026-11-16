use crate::{Field, int, number, text};

/// The text variations whose words aren't prose, so are never learned and, unless the field asks
/// for it, not auto-corrected: a URL (0x10), an e-mail address (0x20), a filter (0xb0) and a web
/// e-mail address (0xd0). URL and e-mail as AOSP; filter as Gboard, which leaves "wprld" alone in
/// one and corrects it in a search box (docs/decisions.md, "Field kinds").
const NOT_PROSE: [i64; 4] = [0x10, 0x20, 0xb0, 0xd0];

/// The address variations among them: a URL, an e-mail address and a web e-mail address.
const ADDRESSES: [i64; 3] = [0x10, 0x20, 0xd0];

/// `TYPE_TEXT_FLAG_AUTO_CORRECT`: the field asks for what is typed in it to be auto-corrected.
/// Gboard corrects an address field that sets it (Contacts' e-mail field, 0x8021), and spaces a
/// word picked there (`spacing::spaces`).
const AUTO_CORRECT: i64 = 0x8000;

/// Full editor identity, including offsets, subtype, settings and privacy. Not a hash:
/// colliding text hashes must never allow a result for a different composing word.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Context(pub Vec<Field>);
impl Context {
    pub fn flag(&self, at: usize) -> bool {
        int(&self.0, at) != 0
    }
    pub fn allowed(&self) -> bool {
        let raw = int(&self.0, 0);
        let password = (raw & 15 == 1 && matches!(raw & 0xff0, 0x80 | 0x90 | 0xe0))
            || (raw & 15 == 2 && raw & 0xff0 == 0x10);
        !password && !(raw & 15 == 1 && raw & 0x80000 != 0)
    }
    pub fn latin(&self) -> bool {
        self.0.get(2) == Some(&text("org.florisboard.nlp.providers.latin"))
            && self.0.get(3) == Some(&text("en"))
            && self.flag(17)
    }
    pub fn enabled(&self) -> bool {
        self.flag(4) || self.flag(5)
    }
    pub fn on(&self) -> bool {
        self.allowed() && self.enabled()
    }
    pub fn words(&self) -> bool {
        self.allowed() && self.flag(4) && self.latin()
    }
    pub fn emoji(&self) -> bool {
        self.allowed() && self.flag(5)
    }
    /// The text variation, if the field is text (`TYPE_CLASS_TEXT`).
    fn text_variation(&self) -> Option<i64> {
        let raw = int(&self.0, 0);
        (raw & 15 == 1).then_some(raw & 0xff0)
    }
    /// Whether the field takes prose, to auto-correct and learn from: not a URL, e-mail address or
    /// filter field ([`NOT_PROSE`]), nor one [`Context::words`] rules out.
    pub fn prose(&self) -> bool {
        let not_prose = self
            .text_variation()
            .is_some_and(|v| NOT_PROSE.contains(&v));
        self.words() && !not_prose
    }
    /// Whether what is typed is auto-corrected: prose, or a URL or e-mail field that asks for it
    /// ([`AUTO_CORRECT`]). A filter field is never corrected, whatever it asks.
    pub fn autocorrect(&self) -> bool {
        let asks = int(&self.0, 0) & AUTO_CORRECT != 0;
        let address = self
            .text_variation()
            .is_some_and(|v| ADDRESSES.contains(&v));
        self.prose() || (self.words() && address && asks)
    }
    /// Whether a word the user kept as typed (by undoing its auto-correction) is remembered across
    /// restarts: only in prose, so never from a password, address or URL field, even one that
    /// asks for corrections, and never incognito.
    pub fn remember_kept(&self) -> bool {
        self.prose() && !self.flag(7)
    }
    /// Whether finished words are learned: only from prose, so nothing typed in an address field
    /// is learned, whatever its flags.
    pub fn learn(&self) -> bool {
        self.prose() && self.flag(6) && !self.flag(7) && !self.flag(1)
    }
    pub fn selection(&self) -> bool {
        int(&self.0, 10) != int(&self.0, 11)
    }
    pub fn query(&self) -> bool {
        self.on() && self.flag(9) && !self.selection()
    }
    pub fn policy(&self) -> Vec<Field> {
        [
            self.on(),
            self.words(),
            self.emoji(),
            self.autocorrect(),
            self.learn(),
            self.enabled(),
        ]
        .map(|v| number(i32::from(v)))
        .to_vec()
    }
    pub fn typing(&self, before: &[u16], typed: &[u16], after: &[u16]) -> Vec<Field> {
        if !self.query()
            || !self.words()
            || typed
                .first()
                .is_some_and(|&u| !chars::is_letter_or_digit(u))
        {
            return vec![];
        }
        let mut before = before.to_vec();
        if typed.is_empty() && before.last().is_some_and(|&u| chars::is_letter_or_digit(u)) {
            before.push(32);
        }
        vec![before, typed.to_vec(), after.to_vec()]
    }
}
