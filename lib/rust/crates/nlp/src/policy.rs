use crate::{Field, int, number, text};

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
    pub fn on(&self) -> bool {
        self.allowed() && (self.flag(4) || self.flag(5))
    }
    pub fn words(&self) -> bool {
        self.allowed() && self.flag(4) && self.latin()
    }
    pub fn emoji(&self) -> bool {
        self.allowed() && self.flag(5)
    }
    pub fn autocorrect(&self) -> bool {
        let raw = int(&self.0, 0);
        self.words() && !(raw & 15 == 1 && matches!(raw & 0xff0, 0x10 | 0x20 | 0xd0))
    }
    pub fn learn(&self) -> bool {
        self.autocorrect() && self.flag(6) && !self.flag(7) && !self.flag(1)
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
