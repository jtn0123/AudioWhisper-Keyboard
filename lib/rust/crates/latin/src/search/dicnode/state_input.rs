//! `suggest/core/dicnode/internal/dic_node_state_input`: how far a DicNode has read the input.
//!
//! `mPrevCodePoint` and `mTerminalDiffCost` are not ported: only `getPrevCodePointG` and
//! `getTerminalDiffCostG`, which the gesture weighting alone calls, read them, so for typing they
//! are written and never read. Nor is the second pointer (`MAX_POINTER_COUNT_G` is 2 for
//! gestures; typing has one).

use crate::defines::MAX_POINTER_COUNT;

/// `DicNodeStateInput`: per pointer, the next input index.
#[derive(Clone, Copy, Debug, Default)]
pub struct DicNodeStateInput {
    /// `mInputIndex`.
    input_index: [i32; MAX_POINTER_COUNT],
}

impl DicNodeStateInput {
    /// `DicNodeStateInput::init()`: for a root.
    pub fn init(&mut self) {
        self.input_index = [0; MAX_POINTER_COUNT];
    }

    /// `DicNodeStateInput::initByCopy`, and `init(src, resetTerminalDiffCost)`, which differs
    /// only in the terminal difference cost.
    pub fn init_by_copy(&mut self, src: &Self) {
        self.input_index = src.input_index;
    }

    /// `DicNodeStateInput::forwardInputIndex`.
    pub fn forward_input_index(&mut self, pointer_id: usize, val: i32) {
        if self.input_index[pointer_id] < 0 {
            self.input_index[pointer_id] = val;
        } else {
            self.input_index[pointer_id] = self.input_index[pointer_id].wrapping_add(val);
        }
    }

    /// `DicNodeStateInput::getInputIndex`: when it is the input's size or more, the node is
    /// completing the word.
    pub fn input_index(&self, pointer_id: usize) -> i32 {
        self.input_index[pointer_id]
    }
}
