//! The probe: node-level reads over a copy of a dictionary body, from `cpp/hooks/trie.cpp`'s
//! `aw_hook_probe_*` functions. Every call holds the oracle's lock.

use super::{collect, length};

/// The node layout the probe writes: 62 ints, as [`ProbedNode::decode`] reads them.
const PROBE_NODE_SLOTS: usize = 3 + 48 + 11;
const PROBE_CODE_POINT_SLOTS: usize = 48;
/// The flags slot, which only the raw reading fills.
const PROBE_FLAGS_SLOT: usize = 3 + PROBE_CODE_POINT_SLOTS + 5;

// SAFETY: cpp/hooks/trie.cpp defines these with the same C signatures; each takes pointers to
// buffers the caller holds for the call.
unsafe extern "C" {
    fn aw_hook_probe_open(body: *const u8, size: usize, table: *const i32, table_size: i32) -> i64;
    fn aw_hook_probe_close(probe: i64);
    fn aw_hook_probe_fetch(probe: i64, pos: i32, out: *mut i32);
    fn aw_hook_probe_read_info(probe: i64, pos: i32, out: *mut i32);
    fn aw_hook_probe_array(probe: i64, pos: i32, out: *mut i32) -> i32;
    fn aw_hook_probe_forward_link(probe: i64, pos: i32, out: *mut i32) -> i32;
    fn aw_hook_probe_bigram_visits(probe: i64, pos: i32, out: *mut i32, capacity: i32) -> i32;
    fn aw_hook_probe_shortcut_entries(
        probe: i64,
        pos: i32,
        max_depth: i32,
        out: *mut i32,
        capacity: i32,
    ) -> i32;
}

/// A node of the trie, as the probe wrote it. Fields the raw reading doesn't fill are zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProbedNode {
    pub valid: bool,
    pub head_pos: i32,
    pub code_points: Vec<i32>,
    pub probability: i32,
    pub children_pos: i32,
    pub shortcut_pos: i32,
    pub bigram_pos: i32,
    pub sibling_pos: i32,
    pub flags: i32,
    pub is_terminal: bool,
    pub is_not_a_word: bool,
    pub is_possibly_offensive: bool,
    pub has_children: bool,
    pub has_bigrams: bool,
}

impl ProbedNode {
    fn decode(slots: &[i32]) -> Self {
        let code_point_count = (slots[2].max(0) as usize).min(PROBE_CODE_POINT_SLOTS);
        let rest = &slots[3 + PROBE_CODE_POINT_SLOTS..];
        Self {
            valid: slots[0] != 0,
            head_pos: slots[1],
            code_points: slots[3..3 + code_point_count].to_vec(),
            probability: rest[0],
            children_pos: rest[1],
            shortcut_pos: rest[2],
            bigram_pos: rest[3],
            sibling_pos: rest[4],
            flags: slots[PROBE_FLAGS_SLOT],
            is_terminal: rest[6] != 0,
            is_not_a_word: rest[7] != 0,
            is_possibly_offensive: rest[8] != 0,
            has_children: rest[9] != 0,
            has_bigrams: rest[10] != 0,
        }
    }
}

/// One entry of a shortcut list, as the probe reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProbedShortcut {
    pub probability: i32,
    pub is_whitelist: bool,
    pub has_next: bool,
    pub code_points: Vec<i32>,
}

/// A node reader over a copy of a dictionary body, made with the C++'s own list policies.
#[derive(Debug)]
pub struct Probe {
    handle: i64,
}

impl Probe {
    /// A probe over a copy of `body`, with `table` as the header's code point table if any.
    pub fn open(body: &[u8], table: Option<&[i32]>) -> Self {
        let _lock = crate::lock();
        let (table_pointer, table_size) = match table {
            Some(table) => (table.as_ptr(), length(table)),
            None => (std::ptr::null(), 0),
        };
        // SAFETY: `body` is valid for its length, and the table (if any) for `table_size` ints. The
        // C++ copies both.
        let handle =
            unsafe { aw_hook_probe_open(body.as_ptr(), body.len(), table_pointer, table_size) };
        Self { handle }
    }

    /// The node at `pos`, as `Ver2ParticiaTrieNodeReader::fetchPtNodeParamsInBufferFromPtNodePos`.
    /// The flags are not reported by that reader, so they are -1.
    pub fn fetch(&self, pos: i32) -> ProbedNode {
        let _lock = crate::lock();
        let mut slots = [0_i32; PROBE_NODE_SLOTS];
        // SAFETY: `slots` holds PROBE_NODE_SLOTS ints, which is all the C++ writes.
        unsafe { aw_hook_probe_fetch(self.handle, pos, slots.as_mut_ptr()) };
        ProbedNode::decode(&slots)
    }

    /// The node at `pos`, as `PatriciaTrieReadingUtils::readPtNodeInfo` gives it, flags included.
    pub fn read_info(&self, pos: i32) -> ProbedNode {
        let _lock = crate::lock();
        let mut slots = [0_i32; PROBE_NODE_SLOTS];
        // SAFETY: `slots` holds PROBE_NODE_SLOTS ints, which is all the C++ writes.
        unsafe { aw_hook_probe_read_info(self.handle, pos, slots.as_mut_ptr()) };
        ProbedNode::decode(&slots)
    }

    /// The node array at `pos`: its count and first node, or `None` when `pos` is outside the body.
    pub fn array(&self, pos: i32) -> Option<(i32, i32)> {
        let _lock = crate::lock();
        let mut out = [0_i32; 2];
        // SAFETY: `out` holds the two ints the C++ writes.
        let valid = unsafe { aw_hook_probe_array(self.handle, pos, out.as_mut_ptr()) };
        (valid != 0).then_some((out[0], out[1]))
    }

    /// The forward link of the node array at `pos`, or `None` when `pos` is outside the body.
    pub fn forward_link(&self, pos: i32) -> Option<i32> {
        let _lock = crate::lock();
        let mut out = [0_i32; 1];
        // SAFETY: `out` holds the one int the C++ writes.
        let valid = unsafe { aw_hook_probe_forward_link(self.handle, pos, out.as_mut_ptr()) };
        (valid != 0).then_some(out[0])
    }

    /// The bigram visits of the list at `pos`, as `BinaryDictionaryBigramsIterator` makes them: the
    /// (probability, bigram position) pairs.
    pub fn bigram_visits(&self, pos: i32) -> Vec<(i32, i32)> {
        let _lock = crate::lock();
        let flat = collect(|out, capacity| {
            // SAFETY: the probe is open, and `out` holds `capacity` ints.
            unsafe { aw_hook_probe_bigram_visits(self.handle, pos, out, capacity) }
        });
        flat.as_chunks::<2>()
            .0
            .iter()
            .map(|&[a, b]| (a, b))
            .collect()
    }

    /// The shortcut entries of the list at `pos`, as `ShortcutListPolicy` reads them: the
    /// probability, whether it is a whitelist, whether another follows, and the target.
    pub fn shortcut_entries(&self, pos: i32, max_depth: usize) -> Vec<ProbedShortcut> {
        let _lock = crate::lock();
        let max_depth = i32::try_from(max_depth).expect("max depth fits");
        let flat = collect(|out, capacity| {
            // SAFETY: the probe is open, and `out` holds `capacity` ints.
            unsafe { aw_hook_probe_shortcut_entries(self.handle, pos, max_depth, out, capacity) }
        });
        let mut entries = Vec::new();
        let mut rest = flat.as_slice();
        while let [probability, whitelist, next, count, tail @ ..] = rest {
            let count = *count as usize;
            entries.push(ProbedShortcut {
                probability: *probability,
                is_whitelist: *whitelist != 0,
                has_next: *next != 0,
                code_points: tail[..count].to_vec(),
            });
            rest = &tail[count..];
        }
        entries
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        let _lock = crate::lock();
        // SAFETY: the handle came from `aw_hook_probe_open` and is closed only here.
        unsafe { aw_hook_probe_close(self.handle) };
    }
}
