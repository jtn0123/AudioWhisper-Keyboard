//! Provider/editor replies resume a delete recipe; the host never chooses its fallback.
use crate::protocol::*;
pub fn plan(v: &[i64]) -> Result<Vec<i64>, &'static str> {
    let [stage, unit, success, composing, forward] = *v else {
        return Err("wrong delete field count");
    };
    if !(0..=1).contains(&unit) {
        return Err("invalid delete unit");
    }
    let mut out = vec![-1];
    if stage == 0 {
        if forward == 0 {
            out.extend([i64::from(DELETE_DROP), 0]);
        }
        for field in [6, 7, 8] {
            out.extend([i64::from(DELETE_STATE), field << 32]);
        }
        if forward != 0 {
            out.extend([
                i64::from(DELETE_NOTIFY),
                0,
                i64::from(DELETE_FORWARD_EDITOR),
                unit,
            ]);
            return Ok(out);
        }
        if unit == 0 {
            out[0] = 1;
            out.extend([i64::from(DELETE_UNDO_AUTO), 0]);
            return Ok(out);
        }
    } else if (stage == 1 || stage == 2) && success != 0 {
        return Ok(out);
    } else if stage == 1 {
        return Ok(vec![2, i64::from(DELETE_UNDO_DICTATION), 0]);
    } else if stage != 2 {
        return Err("invalid delete continuation");
    }
    out.extend([i64::from(DELETE_NOTIFY), 0]);
    if unit == 0 && composing != 0 {
        out.extend([i64::from(DELETE_TAP), 0]);
    }
    out.extend([i64::from(DELETE_BACKWARD_EDITOR), unit]);
    Ok(out)
}
