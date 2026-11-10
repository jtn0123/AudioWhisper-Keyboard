//! `suggest/core/dicnode/internal/dic_node_state_output`: the code points a DicNode will output.
//! For a multi-word suggestion they include the words before the current one.
//!
//! The C++ writes and reads `mOutputCodePoints` without a bounds check where a word reaches
//! `MAX_WORD_LENGTH` (the space after a 48-code-point first word is written one past the array).
//! Here a write past the array is dropped and a read past it is 0.

use crate::defines::{KEYCODE_SPACE, MAX_RESULTS, MAX_WORD_LENGTH, NOT_AN_INDEX};

/// `DicNodeStateOutput`. With the node for "this is a pen": `outputted_code_point_count` is 13
/// (spaces included), `current_word_start` is 10 (where "pen" starts), `prev_word_count` is 3,
/// `prev_words_length` is 10 (the length of "this is a "), `prev_word_start` is 8 (where "a"
/// starts), and `second_word_first_input_index` is where "is" starts in the input.
#[derive(Clone, Copy, Debug)]
pub struct DicNodeStateOutput {
    /// `mOutputtedCodePointCount`.
    outputted_code_point_count: u16,
    /// `mOutputCodePoints`.
    output_code_points: [i32; MAX_WORD_LENGTH],
    /// `mCurrentWordStart`.
    current_word_start: i16,
    /// `mPrevWordCount`.
    prev_word_count: i16,
    /// `mPrevWordsLength`.
    prev_words_length: i16,
    /// `mPrevWordStart`.
    prev_word_start: i16,
    /// `mSecondWordFirstInputIndex`.
    second_word_first_input_index: i32,
}

impl Default for DicNodeStateOutput {
    /// `DicNodeStateOutput()`. The C++ leaves the code points unset; they are 0 here.
    fn default() -> Self {
        Self {
            outputted_code_point_count: 0,
            output_code_points: [0; MAX_WORD_LENGTH],
            current_word_start: 0,
            prev_word_count: 0,
            prev_words_length: 0,
            prev_word_start: 0,
            second_word_first_input_index: NOT_AN_INDEX,
        }
    }
}

impl DicNodeStateOutput {
    /// `DicNodeStateOutput::init()`: for a root.
    pub fn init(&mut self) {
        self.outputted_code_point_count = 0;
        self.current_word_start = 0;
        self.output_code_points[0] = 0;
        self.prev_word_count = 0;
        self.prev_words_length = 0;
        self.prev_word_start = 0;
        self.second_word_first_input_index = NOT_AN_INDEX;
    }

    /// `DicNodeStateOutput::init(stateOutput)`: for the next word, after a space.
    pub fn init_for_next_word(&mut self, state_output: &Self) {
        let count = state_output.outputted_code_point_count;
        self.outputted_code_point_count = count.wrapping_add(1);
        // memmove(mOutputCodePoints, stateOutput->mOutputCodePoints, count * 4)
        let kept = usize::from(count).min(MAX_WORD_LENGTH);
        self.output_code_points[..kept].copy_from_slice(&state_output.output_code_points[..kept]);
        self.set_code_point(usize::from(count), KEYCODE_SPACE);
        self.current_word_start = count.wrapping_add(1) as i16;
        self.prev_word_count = (state_output.prev_word_count + 1).min(MAX_RESULTS as i16);
        self.prev_words_length = count.wrapping_add(1) as i16;
        self.prev_word_start = state_output.current_word_start;
        self.second_word_first_input_index = state_output.second_word_first_input_index;
    }

    /// `DicNodeStateOutput::initByCopy`.
    pub fn init_by_copy(&mut self, state_output: &Self) {
        // memmove(mOutputCodePoints, src, count * 4): only the code points in use are copied.
        let kept = usize::from(state_output.outputted_code_point_count).min(MAX_WORD_LENGTH);
        self.output_code_points[..kept].copy_from_slice(&state_output.output_code_points[..kept]);
        self.outputted_code_point_count = state_output.outputted_code_point_count;
        self.set_code_point(usize::from(self.outputted_code_point_count), 0);
        self.current_word_start = state_output.current_word_start;
        self.prev_word_count = state_output.prev_word_count;
        self.prev_words_length = state_output.prev_words_length;
        self.prev_word_start = state_output.prev_word_start;
        self.second_word_first_input_index = state_output.second_word_first_input_index;
    }

    /// `DicNodeStateOutput::addMergedNodeCodePoints`: appends the code points of a PtNode, as
    /// many as fit.
    pub fn add_merged_node_code_points(&mut self, merged_node_code_points: &[i32]) {
        let count = usize::from(self.outputted_code_point_count);
        let additional = merged_node_code_points
            .len()
            .min(MAX_WORD_LENGTH.saturating_sub(count));
        self.output_code_points[count..count + additional]
            .copy_from_slice(&merged_node_code_points[..additional]);
        self.outputted_code_point_count = (count + additional) as u16;
        self.set_code_point(count + additional, 0);
    }

    /// `output_code_points[index] = value`, for an index inside the array.
    fn set_code_point(&mut self, index: usize, value: i32) {
        if let Some(slot) = self.output_code_points.get_mut(index) {
            *slot = value;
        }
    }

    /// `getCurrentWordCodePointAt`: the code point `index` into the current word.
    pub fn current_word_code_point_at(&self, index: usize) -> i32 {
        usize::try_from(self.current_word_start)
            .ok()
            .and_then(|start| self.output_code_points.get(start + index))
            .copied()
            .unwrap_or(0)
    }

    /// `getCodePointBuf`.
    pub fn code_point_buf(&self) -> &[i32; MAX_WORD_LENGTH] {
        &self.output_code_points
    }

    /// `getSecondWordFirstInputIndex`.
    pub fn second_word_first_input_index(&self) -> i32 {
        self.second_word_first_input_index
    }

    /// `getPrevWordsLength`.
    pub fn prev_words_length(&self) -> i16 {
        self.prev_words_length
    }

    /// `getPrevWordCount`.
    pub fn prev_word_count(&self) -> i16 {
        self.prev_word_count
    }

    /// `getPrevWordStart`.
    pub fn prev_word_start(&self) -> i16 {
        self.prev_word_start
    }
}
