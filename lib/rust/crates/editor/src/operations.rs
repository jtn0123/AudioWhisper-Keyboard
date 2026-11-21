use crate::{Field, Range, number, text};
use wire::editor::effect;

/// Where a plan sets the composing region once the edit's content is known. Never sent: it becomes
/// [`effect::COMPOSING`] or [`effect::FINISH`] first.
pub const NEW_COMPOSING: &str = "newComposing";

#[derive(Clone, Debug)]
pub struct Op {
    pub name: &'static str,
    pub a: i32,
    pub b: i32,
    pub value: Field,
}
impl Op {
    pub fn new(name: &'static str, a: i32, b: i32, value: &[u16]) -> Self {
        Self {
            name,
            a,
            b,
            value: value.to_vec(),
        }
    }
    pub fn simple(name: &'static str) -> Self {
        Self::new(name, 0, 0, &[])
    }
    pub fn composing(range: Range) -> Self {
        if range.valid() {
            Self::new(effect::COMPOSING, range.start, range.end, &[])
        } else {
            Self::simple(effect::FINISH)
        }
    }
    /// This effect as an [`effect::FIELDS`] row.
    pub fn fields(&self) -> Vec<Field> {
        let mut row = vec![vec![]; effect::SIZE];
        row[effect::EFFECT] = text(self.name);
        row[effect::FIRST] = number(self.a);
        row[effect::SECOND] = number(self.b);
        row[effect::VALUE] = self.value.clone();
        row
    }
}

/// The exact modifier press/release order used by the original editor.
pub fn keys(code: i32, meta: i32, count: i32) -> Vec<Op> {
    if count < 1 {
        return vec![];
    }
    let mut ops = vec![Op::simple(effect::BEGIN)];
    for (mask, key) in [(0x1000, 113), (2, 57), (1, 59)] {
        if meta & mask != 0 {
            ops.push(Op::new(effect::DOWN, key, 0, &[]));
        }
    }
    for repeat in 0..count {
        ops.push(Op::new(effect::DOWN, code, meta, &number(repeat)));
    }
    ops.push(Op::new(effect::UP, code, meta, &[]));
    for (mask, key) in [(1, 59), (2, 57), (0x1000, 113)] {
        if meta & mask != 0 {
            ops.push(Op::new(effect::UP, key, 0, &[]));
        }
    }
    ops.push(Op::simple(effect::END));
    ops
}
