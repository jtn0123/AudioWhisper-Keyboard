use crate::{Field, flag_at, int, text};
use wire::flag;
use wire::nlp::{context, policy, typing};

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
    /// Whether [`context`] field `at` is a true flag.
    pub fn flag(&self, at: usize) -> bool {
        flag_at(&self.0, at)
    }
    /// The raw input type.
    fn input_type(&self) -> i64 {
        int(&self.0, context::INPUT_TYPE)
    }
    pub fn allowed(&self) -> bool {
        let raw = self.input_type();
        let password = (raw & 15 == 1 && matches!(raw & 0xff0, 0x80 | 0x90 | 0xe0))
            || (raw & 15 == 2 && raw & 0xff0 == 0x10);
        !password && !(raw & 15 == 1 && raw & 0x80000 != 0)
    }
    pub fn latin(&self) -> bool {
        self.0.get(context::PROVIDER) == Some(&text("org.florisboard.nlp.providers.latin"))
            && self.0.get(context::LANGUAGE) == Some(&text("en"))
            && self.flag(context::READY)
    }
    pub fn enabled(&self) -> bool {
        self.flag(context::SUGGESTIONS) || self.flag(context::EMOJI_SUGGESTIONS)
    }
    pub fn on(&self) -> bool {
        self.allowed() && self.enabled()
    }
    pub fn words(&self) -> bool {
        self.allowed() && self.flag(context::SUGGESTIONS) && self.latin()
    }
    pub fn emoji(&self) -> bool {
        self.allowed() && self.flag(context::EMOJI_SUGGESTIONS)
    }
    /// The text variation, if the field is text (`TYPE_CLASS_TEXT`).
    fn text_variation(&self) -> Option<i64> {
        let raw = self.input_type();
        (raw & 15 == 1).then_some(raw & 0xff0)
    }
    /// Whether the field takes prose, to auto-correct and learn from: not a URL, e-mail address or
    /// filter field (the `NOT_PROSE` variations), nor one [`Context::words`] rules out.
    pub fn prose(&self) -> bool {
        let not_prose = self
            .text_variation()
            .is_some_and(|v| NOT_PROSE.contains(&v));
        self.words() && !not_prose
    }
    /// Whether what is typed is auto-corrected: prose, or a URL or e-mail field that asks for it
    /// (`TYPE_TEXT_FLAG_AUTO_CORRECT`). A filter field is never corrected, whatever it asks.
    pub fn autocorrect(&self) -> bool {
        let asks = self.input_type() & AUTO_CORRECT != 0;
        let address = self
            .text_variation()
            .is_some_and(|v| ADDRESSES.contains(&v));
        self.prose() || (self.words() && address && asks)
    }
    /// Whether a word the user kept as typed (by undoing its auto-correction) is remembered across
    /// restarts: only in prose, so never from a password, address or URL field, even one that
    /// asks for corrections, and never incognito.
    pub fn remember_kept(&self) -> bool {
        self.prose() && !self.flag(context::INCOGNITO)
    }
    /// Whether finished words are learned: only from prose, so nothing typed in an address field
    /// is learned, whatever its flags.
    pub fn learn(&self) -> bool {
        self.prose()
            && self.flag(context::LEARN)
            && !self.flag(context::INCOGNITO)
            && !self.flag(context::RAW_EDITOR)
    }
    pub fn selection(&self) -> bool {
        int(&self.0, context::SELECTION_START) != int(&self.0, context::SELECTION_END)
    }
    pub fn query(&self) -> bool {
        self.on() && self.flag(context::SELECTION_VALID) && !self.selection()
    }
    /// A [`policy`] row.
    pub fn policy(&self) -> Vec<Field> {
        let mut row = vec![vec![]; policy::SIZE];
        row[policy::ON] = flag::utf16(self.on());
        row[policy::WORDS] = flag::utf16(self.words());
        row[policy::EMOJI] = flag::utf16(self.emoji());
        row[policy::AUTOCORRECT] = flag::utf16(self.autocorrect());
        row[policy::LEARN] = flag::utf16(self.learn());
        row[policy::ENABLED] = flag::utf16(self.enabled());
        row
    }
    /// A [`typing`] row as the suggester should see it, or nothing.
    pub fn typing(&self, row: &[Field]) -> Vec<Field> {
        let typed = &row[typing::TYPED];
        if !self.query()
            || !self.words()
            || typed
                .first()
                .is_some_and(|&u| !chars::is_letter_or_digit(u))
        {
            return vec![];
        }
        let mut plan = row.to_vec();
        let before = &mut plan[typing::BEFORE];
        if typed.is_empty() && before.last().is_some_and(|&u| chars::is_letter_or_digit(u)) {
            before.push(32);
        }
        plan
    }
}
