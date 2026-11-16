//! Writing a version 202 dictionary, as AOSP's `BinaryDictEncoderUtils` lays it out.

use super::*;

/// A trie node before layout: its characters, its word, and its children's array.
struct Node {
    chars: Vec<u32>,
    entry: Option<usize>,
    children: Option<usize>,
}

/// The trie laid out: node arrays in the order they are written.
struct Layout {
    arrays: Vec<Vec<Node>>,
    /// Each entry's node, as (array, index).
    nodes_of: Vec<(usize, usize)>,
}

/// A character trie, before runs of single children are merged into one node.
#[derive(Default)]
struct Trie {
    entry: Option<usize>,
    children: Vec<(u32, Trie)>,
}

impl Trie {
    fn insert(&mut self, word: &[u32], entry: usize) -> Result<(), String> {
        let Some((&first, rest)) = word.split_first() else {
            if self.entry.replace(entry).is_some() {
                return Err(format!("{} is in the dictionary twice", entry));
            }
            return Ok(());
        };
        let at = match self.children.binary_search_by_key(&first, |(c, _)| *c) {
            Ok(at) => at,
            Err(at) => {
                self.children.insert(at, (first, Trie::default()));
                at
            }
        };
        self.children[at].1.insert(rest, entry)
    }
}

/// Lays `trie`'s children out as the array at `arrays[index]`, then their children's arrays after
/// it, depth first.
fn lay_out(trie: Trie, layout: &mut Layout, index: usize) {
    let mut subtrees = Vec::new();
    let mut nodes = Vec::with_capacity(trie.children.len());
    for (first, mut child) in trie.children {
        let mut chars = vec![first];
        // A run of nodes that are no word and have one child is one node.
        while child.entry.is_none() && child.children.len() == 1 {
            let (c, grandchild) = child.children.pop().expect("one child");
            chars.push(c);
            child = grandchild;
        }
        if let Some(entry) = child.entry {
            layout.nodes_of[entry] = (index, nodes.len());
        }
        nodes.push(Node {
            chars,
            entry: child.entry,
            children: None,
        });
        subtrees.push(child);
    }
    layout.arrays[index] = nodes;
    for (i, subtree) in subtrees.into_iter().enumerate() {
        if subtree.children.is_empty() {
            continue;
        }
        let child_index = layout.arrays.len();
        layout.arrays.push(Vec::new());
        layout.arrays[index][i].children = Some(child_index);
        lay_out(subtree, layout, child_index);
    }
}

fn code_point_size(c: u32) -> usize {
    if (MIN_ONE_BYTE..=MAX_ONE_BYTE).contains(&c) {
        1
    } else {
        3
    }
}

fn string_size(s: &[u32]) -> usize {
    s.iter().map(|&c| code_point_size(c)).sum::<usize>() + 1
}

const MAX_ADDRESS: usize = 0xFF_FFFF;

fn address_size(offset: usize) -> usize {
    match offset {
        0..=0xFF => 1,
        0x100..=0xFFFF => 2,
        _ => 3,
    }
}

/// The byte sizes of each node's variable fields, refined until the positions they give agree.
struct Sizes {
    /// Per array, per node: the children address size (0 for none).
    children: Vec<Vec<usize>>,
    /// Per entry, per bigram: the address size.
    bigrams: Vec<Vec<usize>>,
}

/// Writes `dictionary` as version 202, laid out as AOSP lays it out.
pub fn write(dictionary: &Dictionary) -> Result<Vec<u8>, String> {
    let entries = &dictionary.entries;
    let mut trie = Trie::default();
    for (i, entry) in entries.iter().enumerate() {
        if entry.word.is_empty() || entry.word.len() > MAX_WORD_LENGTH {
            return Err(format!("entry {i} has {} characters", entry.word.len()));
        }
        if entry
            .word
            .iter()
            .any(|&c| c == u32::from(TERMINATOR) || c > 0x10FFFF)
        {
            return Err(format!("entry {i} has a character the format can't hold"));
        }
        if let Some(&(target, _)) = entry.bigrams.iter().find(|(t, _)| *t >= entries.len()) {
            return Err(format!(
                "entry {i} has a bigram to {target}, which is no entry"
            ));
        }
        trie.insert(&entry.word, i)?;
    }
    let mut layout = Layout {
        arrays: vec![Vec::new()],
        nodes_of: vec![(0, 0); entries.len()],
    };
    lay_out(trie, &mut layout, 0);
    let mut sizes = Sizes {
        children: layout
            .arrays
            .iter()
            .map(|a| {
                a.iter()
                    .map(|n| if n.children.is_some() { 3 } else { 0 })
                    .collect()
            })
            .collect(),
        bigrams: entries.iter().map(|e| vec![3; e.bigrams.len()]).collect(),
    };
    loop {
        let positions = positions(&layout, entries, &sizes);
        if !refine(&layout, entries, &positions, &mut sizes) {
            let body = body(&layout, entries, &positions, &sizes);
            // Every offset is within the body, and the widest address is three bytes.
            if body.len() > MAX_ADDRESS {
                return Err(format!("{} bytes of trie: too big to address", body.len()));
            }
            let mut out = header(dictionary)?;
            out.extend(body);
            return Ok(out);
        }
    }
}

/// Where each array and node starts, given `sizes`.
struct Positions {
    arrays: Vec<usize>,
    nodes: Vec<Vec<usize>>,
}

fn node_size(
    node: &Node,
    entries: &[Entry],
    children_size: usize,
    bigram_sizes: &[usize],
) -> usize {
    let chars = if node.chars.len() == 1 {
        code_point_size(node.chars[0])
    } else {
        string_size(&node.chars)
    };
    let mut size = 1 + chars + children_size;
    if let Some(entry) = node.entry.map(|e| &entries[e]) {
        size += 1;
        if !entry.shortcuts.is_empty() {
            size += 2 + entry
                .shortcuts
                .iter()
                .map(|(t, _)| 1 + string_size(t))
                .sum::<usize>();
        }
        size += bigram_sizes.iter().map(|s| 1 + s).sum::<usize>();
    }
    size
}

fn positions(layout: &Layout, entries: &[Entry], sizes: &Sizes) -> Positions {
    let mut pos = 0;
    let mut out = Positions {
        arrays: Vec::new(),
        nodes: Vec::new(),
    };
    for (a, array) in layout.arrays.iter().enumerate() {
        out.arrays.push(pos);
        pos += if array.len() < 0x80 { 1 } else { 2 };
        let mut nodes = Vec::with_capacity(array.len());
        for (i, node) in array.iter().enumerate() {
            nodes.push(pos);
            let bigrams = node.entry.map_or(&[][..], |e| &sizes.bigrams[e][..]);
            pos += node_size(node, entries, sizes.children[a][i], bigrams);
        }
        out.nodes.push(nodes);
    }
    out
}

/// Where a node's children address field is: after its flags, characters and probability.
fn children_field(node: &Node, at: usize) -> usize {
    let chars = if node.chars.len() == 1 {
        code_point_size(node.chars[0])
    } else {
        string_size(&node.chars)
    };
    at + 1 + chars + usize::from(node.entry.is_some())
}

/// Where each of an entry's bigram address fields is (just after its flags byte).
fn bigram_fields(
    node: &Node,
    entry: &Entry,
    at: usize,
    children_size: usize,
    bigram_sizes: &[usize],
) -> Vec<usize> {
    let mut pos = children_field(node, at) + children_size;
    if !entry.shortcuts.is_empty() {
        pos += 2 + entry
            .shortcuts
            .iter()
            .map(|(t, _)| 1 + string_size(t))
            .sum::<usize>();
    }
    bigram_sizes
        .iter()
        .map(|size| {
            let field = pos + 1;
            pos = field + size;
            field
        })
        .collect()
}

/// Shrinks any size larger than its offset needs; whether any changed.
fn refine(layout: &Layout, entries: &[Entry], positions: &Positions, sizes: &mut Sizes) -> bool {
    let mut changed = false;
    for (a, array) in layout.arrays.iter().enumerate() {
        for (i, node) in array.iter().enumerate() {
            let at = positions.nodes[a][i];
            if let Some(child) = node.children {
                let needed = address_size(positions.arrays[child] - children_field(node, at));
                changed |= needed != sizes.children[a][i];
                sizes.children[a][i] = needed;
            }
            let Some(e) = node.entry else { continue };
            let fields = bigram_fields(
                node,
                &entries[e],
                at,
                sizes.children[a][i],
                &sizes.bigrams[e],
            );
            for (b, (&(target, _), field)) in entries[e].bigrams.iter().zip(fields).enumerate() {
                let (ta, ti) = layout.nodes_of[target];
                let needed = address_size(positions.nodes[ta][ti].abs_diff(field));
                changed |= needed != sizes.bigrams[e][b];
                sizes.bigrams[e][b] = needed;
            }
        }
    }
    changed
}

fn put_uint(out: &mut Vec<u8>, value: usize, size: usize) {
    for i in (0..size).rev() {
        out.push((value >> (8 * i)) as u8);
    }
}

fn put_code_point(out: &mut Vec<u8>, c: u32) {
    if code_point_size(c) == 1 {
        out.push(c as u8);
    } else {
        put_uint(out, c as usize, 3);
    }
}

fn put_string(out: &mut Vec<u8>, s: &[u32]) {
    for &c in s {
        put_code_point(out, c);
    }
    out.push(TERMINATOR);
}

fn header(dictionary: &Dictionary) -> Result<Vec<u8>, String> {
    let mut attributes = Vec::new();
    for (key, value) in &dictionary.attributes {
        put_string(&mut attributes, &code_points(key));
        put_string(&mut attributes, &code_points(value));
    }
    let size = 12 + attributes.len();
    let mut out = Vec::with_capacity(size);
    put_uint(&mut out, MAGIC as usize, 4);
    put_uint(&mut out, usize::from(VERSION), 2);
    put_uint(&mut out, usize::from(dictionary.flags), 2);
    put_uint(&mut out, size, 4);
    out.extend(attributes);
    Ok(out)
}

fn body(layout: &Layout, entries: &[Entry], positions: &Positions, sizes: &Sizes) -> Vec<u8> {
    let mut out = Vec::new();
    for (a, array) in layout.arrays.iter().enumerate() {
        debug_assert_eq!(out.len(), positions.arrays[a]);
        if array.len() < 0x80 {
            out.push(array.len() as u8);
        } else {
            put_uint(&mut out, array.len() | 0x8000, 2);
        }
        for (i, node) in array.iter().enumerate() {
            let at = out.len();
            debug_assert_eq!(at, positions.nodes[a][i]);
            let entry = node.entry.map(|e| &entries[e]);
            let children_size = sizes.children[a][i];
            let mut flags = (children_size as u8) << 6;
            if node.chars.len() > 1 {
                flags |= MULTIPLE_CHARS;
            }
            if let Some(entry) = entry {
                flags |= TERMINAL;
                flags |= if entry.shortcuts.is_empty() {
                    0
                } else {
                    HAS_SHORTCUTS
                };
                flags |= if entry.bigrams.is_empty() {
                    0
                } else {
                    HAS_BIGRAMS
                };
                flags |= if entry.not_a_word { NOT_A_WORD } else { 0 };
                flags |= if entry.possibly_offensive {
                    POSSIBLY_OFFENSIVE
                } else {
                    0
                };
            }
            out.push(flags);
            if node.chars.len() == 1 {
                put_code_point(&mut out, node.chars[0]);
            } else {
                put_string(&mut out, &node.chars);
            }
            if let Some(entry) = entry {
                out.push(entry.probability);
            }
            if let Some(child) = node.children {
                let offset = positions.arrays[child] - out.len();
                put_uint(&mut out, offset, children_size);
            }
            let Some(entry) = entry else { continue };
            if !entry.shortcuts.is_empty() {
                let size = 2 + entry
                    .shortcuts
                    .iter()
                    .map(|(t, _)| 1 + string_size(t))
                    .sum::<usize>();
                put_uint(&mut out, size, 2);
                for (k, (target, probability)) in entry.shortcuts.iter().enumerate() {
                    let next = if k + 1 < entry.shortcuts.len() {
                        ATTRIBUTE_HAS_NEXT
                    } else {
                        0
                    };
                    out.push(next | (probability & PROBABILITY_MASK));
                    put_string(&mut out, target);
                }
            }
            let e = node.entry.unwrap_or_default();
            for (b, &(target, probability)) in entry.bigrams.iter().enumerate() {
                let (ta, ti) = layout.nodes_of[target];
                let target_pos = positions.nodes[ta][ti];
                let field = out.len() + 1;
                let size = sizes.bigrams[e][b];
                let mut flags = (size as u8) << 4 | (probability & PROBABILITY_MASK);
                if b + 1 < entry.bigrams.len() {
                    flags |= ATTRIBUTE_HAS_NEXT;
                }
                if target_pos < field {
                    flags |= BIGRAM_OFFSET_NEGATIVE;
                }
                out.push(flags);
                put_uint(&mut out, target_pos.abs_diff(field), size);
            }
        }
    }
    out
}
