//! Editor operation routing. Key/domain codes have one canonical definition in `keys`.
use crate::{Field, Session, integer, operations::keys as key_events};
use keys::codes;
use wire::editor as ops;
use wire::flag;
impl Session {
    pub(crate) fn arrow(&mut self, fields: &[Field]) -> Vec<Field> {
        let (code, alt) = match integer(fields, 0) {
            codes::ARROW_LEFT => (21, false),
            codes::ARROW_RIGHT => (22, false),
            codes::ARROW_UP => (19, false),
            codes::ARROW_DOWN => (20, false),
            codes::MOVE_START_OF_PAGE => (19, true),
            codes::MOVE_END_OF_PAGE => (20, true),
            codes::MOVE_START_OF_LINE => (21, true),
            codes::MOVE_END_OF_LINE => (22, true),
            _ => return self.done(false, vec![]),
        };
        let meta = (if alt { 0x12 } else { 0 }) | (if flag::decode(&fields[2]) { 0x41 } else { 0 });
        self.done(
            integer(fields, 1) > 0,
            key_events(code, meta, integer(fields, 1)),
        )
    }
    pub(crate) fn mass_state(&mut self, operation: &str) -> Vec<Field> {
        match operation {
            ops::MASS_BEGIN => {
                self.mass = self.mass.wrapping_add(1);
                vec![]
            }
            ops::MASS_END => {
                self.mass = self.mass.wrapping_sub(1);
                vec![flag::utf16(self.mass == 0)]
            }
            ops::MASS_RESET => {
                self.mass = 0;
                vec![]
            }
            _ => vec![flag::utf16(self.mass > 0)],
        }
    }
}
