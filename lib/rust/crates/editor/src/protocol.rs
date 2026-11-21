//! Validate public messages before dispatch; malformed JNI arrays become Java exceptions.
use crate::{Field, Session};
use wire::editor::{self as ops, char_input, config, context, cursor, extracted};
impl Session {
    pub fn checked_call(
        &mut self,
        operation: &str,
        fields: &[Field],
    ) -> Result<Vec<Field>, &'static str> {
        if operation == ops::COMPOSER_RULES {
            if !fields.len().is_multiple_of(2) {
                return Err("incomplete composer rule");
            }
            return Ok(self.call(operation, fields));
        }
        let expected = match operation {
            ops::INPUT_INFO | ops::UNAVAILABLE | ops::RAW_ENTER => 1,
            ops::ARROW => 3,
            ops::EXTRACTED => extracted::SIZE,
            ops::CONFIG => config::SIZE,
            ops::CONTEXT => context::SIZE,
            ops::START => 3,
            ops::UPDATE => cursor::SIZE + 1,
            ops::MASS | ops::LATE => cursor::SIZE,
            ops::READ => 3,
            ops::WORD => 2,
            ops::CAPS | ops::CHARACTER | ops::MEASURE | ops::EXTENT => 1,
            ops::COMPOSER | ops::SPACING => 2,
            ops::CHAR => char_input::SIZE,
            ops::COMMIT
            | ops::FINALIZE
            | ops::TYPE_CHAR
            | ops::TYPE_TEXT
            | ops::COMPLETION
            | ops::GESTURE
            | ops::BACKWARD
            | ops::FORWARD
            | ops::ACTION => 1,
            ops::REPLACE | ops::SELECT | ops::DICTATION => 2,
            ops::DELETE | ops::KEYS | ops::REVERT | ops::SELECT_AROUND => 3,
            ops::RESULTS => self.operations.len(),
            ops::MASS_BEGIN
            | ops::MASS_END
            | ops::MASS_STATE
            | ops::MASS_RESET
            | ops::DOUBLE_SPACE => 0,
            ops::PUBLISHED
            | ops::SNAPSHOT
            | ops::RESET
            | ops::LAST_COMMIT
            | ops::LAST_DELETE
            | ops::REFRESH
            | ops::SPACES
            | ops::UNDO_DICTATION
            | ops::FORGET_DICTATION
            | ops::ENTER
            | ops::SELECT_ALL => 0,
            _ => return Err("unknown editor operation"),
        };
        if fields.len() != expected {
            return Err("wrong editor field count");
        }
        if matches!(operation, ops::WORD | ops::CAPS) && self.pending.is_none() {
            return Err("no pending editor analysis");
        }
        if matches!(
            operation,
            ops::READ | ops::MEASURE | ops::COMPOSER | ops::CHARACTER
        ) && matches!(self.query, crate::session::Query::None)
        {
            return Err("no pending editor query");
        }
        if matches!(operation, ops::SPACING | ops::EXTENT) && self.context.event.is_none() {
            return Err("no pending editor event");
        }
        Ok(self.call(operation, fields))
    }
}
