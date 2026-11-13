//! Complete state/subtype/space decisions around Android callbacks and editor/provider replies.
use crate::protocol::*;
use keys::models;
pub fn plan(v: &[i64], text: &[u16]) -> Result<Vec<i64>, &'static str> {
    let Some((&operation, args)) = v.split_first() else {
        return Err("empty control packet");
    };
    match operation as i32 {
        CONTROL_DELETE => crate::delete::plan(args),
        CONTROL_FIND_SUBTYPE if !args.is_empty() => Ok(vec![
            args[1..]
                .iter()
                .position(|id| *id == args[0])
                .map_or(-1, |i| i as i64),
        ]),
        CONTROL_EVALUATOR if args.len() == 1 => {
            let raw = args[0] as u64;
            Ok(vec![if models::read(raw, 0) == Some(0) {
                raw
            } else {
                models::write(raw, 2, 0).expect("shift")
            } as i64])
        }
        CONTROL_SUBTYPE if args.len() >= 2 => {
            let active = args[0];
            let default = args[1];
            let ids = &args[2..];
            let index = ids
                .iter()
                .position(|id| *id == active)
                .or(if ids.is_empty() { None } else { Some(0) });
            let chosen = index.map_or(default, |i| ids[i]);
            Ok(vec![
                index.map_or(-1, |i| i as i64),
                i64::from(chosen != active),
            ])
        }
        CONTROL_CYCLE if !args.is_empty() => {
            let reverse = args[0] < 0;
            let flags = &args[1..];
            let mut trigger = false;
            let mut selected = -1;
            let mut order: Vec<usize> = (0..flags.len()).collect();
            if reverse {
                order.reverse();
            }
            for &i in &order {
                if trigger {
                    trigger = false;
                    selected = i as i64;
                } else if flags[i] != 0 {
                    trigger = true;
                }
            }
            if trigger {
                selected = order[0] as i64;
            }
            Ok(vec![selected])
        }
        CONTROL_SPACE if args.len() == 6 => {
            let mode = if args[1] != 0 && matches!(args[0], 2 | 3 | 5) {
                0
            } else {
                args[0]
            };
            let period = args[2] != 0 && args[3] != 0 && shift::is_double_space_period(text);
            let space = !period && (args[4] != 0 || args[5] == 0);
            Ok(vec![mode, i64::from(period), i64::from(space)])
        }
        CONTROL_START if args.len() == 4 => {
            let mut raw = args[0] as u64;
            if models::read(raw, 3) != Some(2) || args[1] != 0 {
                raw = models::write(raw, 3, 0).expect("UI mode");
            }
            raw = models::write(
                raw,
                5,
                i32::from(args[2] >= 0 && args[3] >= 0 && args[2] != args[3]),
            )
            .expect("selection");
            Ok(vec![raw as i64])
        }
        CONTROL_FINISH if args.len() == 1 => {
            let mut raw = args[0] as u64;
            for field in [3, 10, 11] {
                raw = models::write(raw, field, 0).expect("state field");
            }
            Ok(vec![raw as i64])
        }
        _ => Err("wrong control operation or field count"),
    }
}
