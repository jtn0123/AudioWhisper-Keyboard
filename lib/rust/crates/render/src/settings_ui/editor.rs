//! UTF-16 editable state. Android is only the InputConnection transport.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Editor {
    pub text: Vec<u16>,
    pub anchor: usize,
    pub focus: usize,
    pub composing: Option<(usize, usize)>,
    pub focused: bool,
}

/// The keyboard's batch edits on the field (`InputConnection.beginBatchEdit`/`endBatchEdit`),
/// however deeply nested. While one is open the app holds back the cursor updates it sends the
/// keyboard, as EditText does: the keyboard then sees one update per batch, the state its edit
/// leaves, not each step of it, which it would take for the user moving the cursor (and end the
/// space it owes after undoing an auto-correction).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Batches(u32);
impl Batches {
    pub fn begin(&mut self) {
        self.0 = self.0.saturating_add(1);
    }
    /// Ends the innermost batch; whether one is still open, which is `endBatchEdit`'s answer.
    pub fn end(&mut self) -> bool {
        self.0 = self.0.saturating_sub(1);
        self.0 > 0
    }
    /// A closed connection ends the batches it left open, as `EditableInputConnection` does.
    pub fn close(&mut self) {
        self.0 = 0;
    }
    /// Whether a change to the field is reported to the keyboard now.
    pub fn report(&self) -> bool {
        self.0 == 0
    }
}

impl Editor {
    pub fn selection(&self) -> (usize, usize) {
        (self.anchor.min(self.focus), self.anchor.max(self.focus))
    }
    pub fn select(&mut self, anchor: i32, focus: i32) -> bool {
        if anchor < 0
            || focus < 0
            || anchor as usize > self.text.len()
            || focus as usize > self.text.len()
        {
            return false;
        }
        self.anchor = anchor.min(focus) as usize;
        self.focus = anchor.max(focus) as usize;
        true
    }
    pub fn replace(&mut self, text: &[u16], cursor: i32, composing: bool) -> bool {
        let (start, end) = self.composing.unwrap_or_else(|| self.selection());
        if self.text.len() - (end - start) + text.len() > 65_536 {
            return false;
        }
        self.text.splice(start..end, text.iter().copied());
        let new = if cursor > 0 {
            start as i64 + text.len() as i64 + cursor as i64 - 1
        } else {
            start as i64 + cursor as i64
        };
        self.anchor = new.clamp(0, self.text.len() as i64) as usize;
        self.focus = self.anchor;
        self.composing = (composing && !text.is_empty()).then_some((start, start + text.len()));
        true
    }
    pub fn region(&mut self, start: i32, end: i32) {
        let a = start.min(end).clamp(0, self.text.len() as i32) as usize;
        let b = start.max(end).clamp(0, self.text.len() as i32) as usize;
        self.composing = (a != b).then_some((a, b));
    }
    fn delete_range(&mut self, start: usize, end: usize) {
        self.text.drain(start..end);
        let map = |n: usize| {
            if n >= end {
                n - (end - start)
            } else {
                n.min(start)
            }
        };
        self.anchor = map(self.anchor);
        self.focus = map(self.focus);
        if let Some((a, b)) = self.composing {
            let (a, b) = (map(a), map(b));
            self.composing = (a != b).then_some((a, b));
        }
    }
    pub fn delete(&mut self, before: i32, after: i32, codepoints: bool) -> bool {
        if before < 0 || after < 0 {
            return false;
        }
        // Compose's EditCommands delete around selection, not around the composing span.
        // An unmatched surrogate is one unit; only a genuine pair consumes two.
        let (start, end) = self.selection();
        let mut left = start;
        let mut right = end;
        if codepoints {
            for _ in 0..before.min(self.text.len() as i32) {
                if left == 0 {
                    break;
                }
                left -= 1;
                if left > 0
                    && (0xdc00..=0xdfff).contains(&self.text[left])
                    && (0xd800..=0xdbff).contains(&self.text[left - 1])
                {
                    left -= 1;
                }
            }
            for _ in 0..after.min(self.text.len() as i32) {
                if right == self.text.len() {
                    break;
                }
                let pair = right + 1 < self.text.len()
                    && (0xd800..=0xdbff).contains(&self.text[right])
                    && (0xdc00..=0xdfff).contains(&self.text[right + 1]);
                right += if pair { 2 } else { 1 };
            }
        } else {
            left = start.saturating_sub(before as usize);
            right = end.saturating_add(after as usize).min(self.text.len());
        }
        self.delete_range(end, right);
        self.delete_range(left, start);
        true
    }
    pub fn backspace(&mut self) {
        self.composing = None;
        if self.anchor != self.focus {
            self.replace(&[], 1, false);
        } else {
            self.delete(1, 0, true);
        }
    }
    pub fn query(&self, what: &str, length: usize) -> Vec<u16> {
        let (a, b) = self.selection();
        match what {
            "before" => self.text[a.saturating_sub(length)..a].to_vec(),
            "after" => self.text[b..b.saturating_add(length).min(self.text.len())].to_vec(),
            "selected" => self.text[a..b].to_vec(),
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Batches;

    #[test]
    fn the_cursor_is_reported_only_once_the_outermost_batch_ends() {
        let mut batches = Batches::default();
        assert!(batches.report());
        batches.begin();
        batches.begin();
        assert!(!batches.report());
        assert!(batches.end(), "one is still open");
        assert!(!batches.report());
        assert!(!batches.end(), "none is open");
        assert!(batches.report());
        // An unmatched end opens nothing.
        assert!(!batches.end());
        assert!(batches.report());
    }

    #[test]
    fn a_closed_connection_ends_the_batches_it_left_open() {
        let mut batches = Batches::default();
        batches.begin();
        batches.begin();
        batches.close();
        assert!(batches.report());
        batches.begin();
        assert!(!batches.end());
    }
}
