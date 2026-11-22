//! What a tape records: each answer's canonical words, hashed, and a short text of it for the
//! golden file's samples.

use std::fmt::Write as _;

/// FNV-1a over 64-bit words. Each step (xor, then multiply by an odd number) is a bijection of
/// the state, so two streams that differ in one word always end in different states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fnv(u64);

impl Default for Fnv {
    fn default() -> Self {
        Self(0xCBF2_9CE4_8422_2325)
    }
}

impl Fnv {
    const PRIME: u64 = 0x0000_0100_0000_01B3;

    pub fn word(&mut self, word: u64) {
        self.0 = (self.0 ^ word).wrapping_mul(Self::PRIME);
    }

    pub fn finish(self) -> u64 {
        self.0
    }
}

/// A value a test records: the words it hashes to, and its text in a sample.
pub trait Answer {
    fn feed(&self, fnv: &mut Fnv);
    fn render(&self, out: &mut String);
}

/// A signed number sign-extended, so -1 is the same word whatever its width.
fn signed(it: impl Into<i64>) -> u64 {
    it.into() as u64
}

fn unsigned(it: impl Into<u64>) -> u64 {
    it.into()
}

macro_rules! integers {
    ($($kind:ty => $word:ident),*) => {$(
        impl Answer for $kind {
            fn feed(&self, fnv: &mut Fnv) {
                fnv.word($word(*self));
            }
            fn render(&self, out: &mut String) {
                let _ = write!(out, "{self}");
            }
        }
    )*};
}

integers!(i8 => signed, i16 => signed, i32 => signed, i64 => signed);
integers!(u8 => unsigned, u16 => unsigned, u32 => unsigned, u64 => unsigned);

impl Answer for usize {
    fn feed(&self, fnv: &mut Fnv) {
        fnv.word(*self as u64);
    }
    fn render(&self, out: &mut String) {
        let _ = write!(out, "{self}");
    }
}

impl Answer for bool {
    fn feed(&self, fnv: &mut Fnv) {
        fnv.word(u64::from(*self));
    }
    fn render(&self, out: &mut String) {
        let _ = write!(out, "{self}");
    }
}

impl Answer for char {
    fn feed(&self, fnv: &mut Fnv) {
        fnv.word(u64::from(*self));
    }
    fn render(&self, out: &mut String) {
        let _ = write!(out, "{self:?}");
    }
}

/// A float by its bits: NaNs and signed zeros as the C++ left them.
impl Answer for f32 {
    fn feed(&self, fnv: &mut Fnv) {
        fnv.word(u64::from(self.to_bits()));
    }
    fn render(&self, out: &mut String) {
        let _ = write!(out, "{self:?}#{:08x}", self.to_bits());
    }
}

impl Answer for str {
    fn feed(&self, fnv: &mut Fnv) {
        fnv.word(self.len() as u64);
        self.bytes().for_each(|it| fnv.word(u64::from(it)));
    }
    fn render(&self, out: &mut String) {
        let _ = write!(out, "{self:?}");
    }
}

impl Answer for String {
    fn feed(&self, fnv: &mut Fnv) {
        self.as_str().feed(fnv);
    }
    fn render(&self, out: &mut String) {
        self.as_str().render(out);
    }
}

impl<T: Answer> Answer for [T] {
    fn feed(&self, fnv: &mut Fnv) {
        fnv.word(self.len() as u64);
        self.iter().for_each(|it| it.feed(fnv));
    }
    fn render(&self, out: &mut String) {
        out.push('[');
        for (index, item) in self.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            item.render(out);
        }
        out.push(']');
    }
}

impl<T: Answer, const N: usize> Answer for [T; N] {
    fn feed(&self, fnv: &mut Fnv) {
        self.as_slice().feed(fnv);
    }
    fn render(&self, out: &mut String) {
        self.as_slice().render(out);
    }
}

impl<T: Answer> Answer for Vec<T> {
    fn feed(&self, fnv: &mut Fnv) {
        self.as_slice().feed(fnv);
    }
    fn render(&self, out: &mut String) {
        self.as_slice().render(out);
    }
}

impl<T: Answer> Answer for Option<T> {
    fn feed(&self, fnv: &mut Fnv) {
        match self {
            None => fnv.word(0),
            Some(it) => {
                fnv.word(1);
                it.feed(fnv);
            }
        }
    }
    fn render(&self, out: &mut String) {
        match self {
            None => out.push_str("None"),
            Some(it) => it.render(out),
        }
    }
}

impl<T: Answer + ?Sized> Answer for &T {
    fn feed(&self, fnv: &mut Fnv) {
        (**self).feed(fnv);
    }
    fn render(&self, out: &mut String) {
        (**self).render(out);
    }
}

macro_rules! tuples {
    ($(($($name:ident $index:tt),+)),*) => {$(
        impl<$($name: Answer),+> Answer for ($($name,)+) {
            fn feed(&self, fnv: &mut Fnv) {
                $(self.$index.feed(fnv);)+
            }
            fn render(&self, out: &mut String) {
                out.push('(');
                $(
                    if $index > 0 {
                        out.push(',');
                    }
                    self.$index.render(out);
                )+
                out.push(')');
            }
        }
    )*};
}

tuples!(
    (A 0, B 1),
    (A 0, B 1, C 2),
    (A 0, B 1, C 2, D 3),
    (A 0, B 1, C 2, D 3, E 4),
    (A 0, B 1, C 2, D 3, E 4, F 5),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7)
);

/// What the app reads from one decode: each suggestion's code points up to its terminator, with
/// its score, type and space index, the auto-commit confidence when there is a suggestion, and
/// the weight's bits. `suggest`'s session goldens record this.
#[derive(Debug)]
pub struct Suggestions<'a> {
    pub count: i32,
    pub code_points: &'a [i32],
    pub scores: &'a [i32],
    pub types: &'a [i32],
    pub space_indices: &'a [i32],
    pub auto_commit_confidence: i32,
    pub weight: f32,
}

impl Suggestions<'_> {
    fn count(&self) -> usize {
        usize::try_from(self.count).unwrap_or(0)
    }

    fn word(&self, slot: usize) -> &[i32] {
        let word = &self.code_points[slot * 48..(slot + 1) * 48];
        &word[..word.iter().position(|&it| it == 0).unwrap_or(48)]
    }
}

impl Answer for Suggestions<'_> {
    fn feed(&self, fnv: &mut Fnv) {
        self.count.feed(fnv);
        let count = self.count();
        for slot in 0..count {
            self.word(slot).feed(fnv);
        }
        self.scores[..count].feed(fnv);
        self.types[..count].feed(fnv);
        self.space_indices[..count].feed(fnv);
        if count > 0 {
            self.auto_commit_confidence.feed(fnv);
        }
        self.weight.feed(fnv);
    }

    fn render(&self, out: &mut String) {
        let count = self.count();
        let _ = write!(out, "{count}:");
        for slot in 0..count {
            out.push(' ');
            self.word(slot).render(out);
            let _ = write!(
                out,
                "/{}/{}/{}",
                self.scores[slot], self.types[slot], self.space_indices[slot]
            );
        }
        let _ = write!(
            out,
            " c{} w{:08x}",
            self.auto_commit_confidence,
            self.weight.to_bits()
        );
    }
}
