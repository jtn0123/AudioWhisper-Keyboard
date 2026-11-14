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
            let (index, changed) = settings::subtype::active(&args[2..], args[0], args[1]);
            Ok(vec![index, i64::from(changed)])
        }
        CONTROL_CYCLE if !args.is_empty() => {
            Ok(vec![settings::subtype::cycle(&args[1..], args[0] < 0)])
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
