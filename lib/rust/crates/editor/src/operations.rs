use crate::{Field, Range, number, text};

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
            Self::new("composing", range.start, range.end, &[])
        } else {
            Self::simple("finish")
        }
    }
    pub fn fields(&self) -> Vec<Field> {
        vec![
            text(self.name),
            number(self.a),
            number(self.b),
            self.value.clone(),
        ]
    }
}

/// The exact modifier press/release order used by the original editor.
pub fn keys(code: i32, meta: i32, count: i32) -> Vec<Op> {
    if count < 1 {
        return vec![];
    }
    let mut ops = vec![Op::simple("begin")];
    for (mask, key) in [(0x1000, 113), (2, 57), (1, 59)] {
        if meta & mask != 0 {
            ops.push(Op::new("down", key, 0, &[]));
        }
    }
    for repeat in 0..count {
        ops.push(Op::new("down", code, meta, &number(repeat)));
    }
    ops.push(Op::new("up", code, meta, &[]));
    for (mask, key) in [(1, 59), (2, 57), (0x1000, 113)] {
        if meta & mask != 0 {
            ops.push(Op::new("up", key, 0, &[]));
        }
    }
    ops.push(Op::simple("end"));
    ops
}
