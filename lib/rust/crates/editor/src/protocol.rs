//! Validate public messages before dispatch; malformed JNI arrays become Java exceptions.
use crate::{Field, Session};
impl Session {
    pub fn checked_call(
        &mut self,
        operation: &str,
        fields: &[Field],
    ) -> Result<Vec<Field>, &'static str> {
        let expected = match operation {
            "inputInfo" | "unavailable" | "rawEnter" => 1,
            "arrow" => 3,
            "extracted" => 8,
            "config" => 5,
            "context" => 10,
            "start" => 3,
            "update" => 5,
            "mass" => 4,
            "read" => 3,
            "word" => 2,
            "caps" | "character" | "measure" | "extent" => 1,
            "composer" | "spacing" => 2,
            "char" => 4,
            "commit" | "finalize" | "typeChar" | "typeText" | "completion" | "gesture"
            | "backward" | "forward" | "action" => 1,
            "replace" | "select" | "dictation" => 2,
            "delete" | "keys" | "revert" | "selectAround" => 3,
            "results" => self.operations.len(),
            "massBegin" | "massEnd" | "massState" | "massReset" | "doubleSpace" => 0,
            "published" | "snapshot" | "reset" | "lastCommit" | "lastDelete" | "refresh"
            | "spaces" | "undoDictation" | "forgetDictation" | "enter" | "selectAll" => 0,
            _ => return Err("unknown editor operation"),
        };
        if fields.len() != expected {
            return Err("wrong editor field count");
        }
        if matches!(operation, "word" | "caps") && self.pending.is_none() {
            return Err("no pending editor analysis");
        }
        if matches!(operation, "read" | "measure" | "composer" | "character")
            && matches!(self.query, crate::session::Query::None)
        {
            return Err("no pending editor query");
        }
        if matches!(operation, "spacing" | "extent") && self.context.event.is_none() {
            return Err("no pending editor event");
        }
        Ok(self.call(operation, fields))
    }
}
