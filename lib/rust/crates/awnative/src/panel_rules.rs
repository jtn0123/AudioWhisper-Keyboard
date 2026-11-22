//! Shared preference policy JNI. All returned ids refer to the caller's value pool.
use render::arrangement::{Arrangement, Editor};

use crate::jni::{Result, illegal};

pub fn history(pinned: &[i32], recent: &[i32], c: &[i32]) -> Result<Vec<Vec<i32>>> {
    let [enabled, ps, rs, pl, rl, op, value, offset] = c else {
        return Err(illegal("history policy length"));
    };
    if !(0..4).contains(ps) || !(0..4).contains(rs) || *pl < 0 || *rl < 0 {
        return Err(illegal("history policy range"));
    }
    let mut h = emoji::history::History {
        pinned: pinned.to_vec(),
        recent: recent.to_vec(),
    };
    h.apply(
        emoji::history::Policy {
            enabled: *enabled != 0,
            pinned: *ps,
            recent: *rs,
            pinned_limit: *pl,
            recent_limit: *rl,
        },
        *op,
        *value,
        *offset,
    );
    Ok(vec![h.pinned, h.recent])
}
pub fn arrange(c: &[i32], dynamic: &[i32], hidden: &[i32]) -> Result<Vec<Vec<i32>>> {
    if c.is_empty() || !(c.len() - 1).is_multiple_of(3) {
        return Err(illegal("arrangement operations length"));
    }
    let mut a = Arrangement {
        sticky: c[0],
        dynamic: dynamic.to_vec(),
        hidden: hidden.to_vec(),
    };
    let mut e = Editor::new(a.clone());
    for op in c[1..].as_chunks::<3>().0 {
        match op[0] {
            0 => {
                a.distinct();
                e = Editor::new(a.clone());
            }
            1 => e.begin(op[1], op[2] as usize),
            2 => e.over(op[1], op[2] as usize),
            3 => e.complete(),
            _ => return Err(illegal("unknown arrangement operation")),
        }
    }
    let saved = e.saved();
    Ok(vec![vec![saved.sticky], saved.dynamic, saved.hidden])
}
