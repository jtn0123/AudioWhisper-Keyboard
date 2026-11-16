//! Reading a version 202 dictionary back whole.

use super::*;
use std::collections::HashMap;

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn u8(&mut self) -> Result<u8, String> {
        let byte = *self
            .bytes
            .get(self.pos)
            .ok_or("the dictionary ends early")?;
        self.pos += 1;
        Ok(byte)
    }

    fn uint(&mut self, size: usize) -> Result<u32, String> {
        let mut value = 0;
        for _ in 0..size {
            value = value << 8 | u32::from(self.u8()?);
        }
        Ok(value)
    }

    /// A code point, or `None` at a string's terminator.
    fn code_point(&mut self) -> Result<Option<u32>, String> {
        let first = self.u8()?;
        if u32::from(first) >= MIN_ONE_BYTE {
            Ok(Some(u32::from(first)))
        } else if first == TERMINATOR {
            Ok(None)
        } else {
            Ok(Some(u32::from(first) << 16 | self.uint(2)?))
        }
    }

    fn string(&mut self) -> Result<Vec<u32>, String> {
        let mut out = Vec::new();
        while let Some(c) = self.code_point()? {
            out.push(c);
            if out.len() > 4 * MAX_WORD_LENGTH {
                return Err("a string runs on".to_string());
            }
        }
        Ok(out)
    }
}

/// A node as read: where it is, and its bigrams' target positions.
struct ReadNode {
    entry: Option<usize>,
    bigrams: Vec<(usize, u8)>,
}

/// Reads a whole version 202 dictionary.
pub fn read(bytes: &[u8]) -> Result<Dictionary, String> {
    let mut r = Reader { bytes, pos: 0 };
    if r.uint(4)? != MAGIC {
        return Err("not a LatinIME dictionary".to_string());
    }
    let version = r.uint(2)? as u16;
    if version != VERSION {
        return Err(format!("version {version}, not {VERSION}"));
    }
    let flags = r.uint(2)? as u16;
    let header_size = r.uint(4)? as usize;
    let mut attributes = Vec::new();
    while r.pos < header_size {
        let key = string(&r.string()?);
        let value = string(&r.string()?);
        attributes.push((key, value));
    }
    let body = bytes
        .get(header_size..)
        .ok_or("the header runs past the end")?;
    let mut walk = Walk {
        r: Reader {
            bytes: body,
            pos: 0,
        },
        entries: Vec::new(),
        nodes: HashMap::new(),
    };
    walk.array(&mut Vec::new(), 0)?;
    let Walk { entries, nodes, .. } = walk;
    let mut entries = entries;
    for node in nodes.values() {
        let Some(index) = node.entry else { continue };
        for &(target, probability) in &node.bigrams {
            let target = nodes
                .get(&target)
                .and_then(|n| n.entry)
                .ok_or(format!("a bigram points at {target}, which is no word"))?;
            entries[index].bigrams.push((target, probability));
        }
    }
    Ok(Dictionary {
        flags,
        attributes,
        entries,
    })
}

struct Walk<'a> {
    r: Reader<'a>,
    entries: Vec<Entry>,
    /// Each node's position, with what it is.
    nodes: HashMap<usize, ReadNode>,
}

impl Walk<'_> {
    /// The node array at the reader's position, and everything under it, with `prefix` before
    /// each of its words.
    fn array(&mut self, prefix: &mut Vec<u32>, depth: usize) -> Result<(), String> {
        if depth > MAX_WORD_LENGTH {
            return Err("the trie is too deep".to_string());
        }
        let first = self.r.u8()?;
        let count = if first < 0x80 {
            usize::from(first)
        } else {
            usize::from(first & 0x7F) << 8 | usize::from(self.r.u8()?)
        };
        let mut children = Vec::new();
        for _ in 0..count {
            let start = self.r.pos;
            let (chars, child) = self.node(prefix)?;
            if let Some(child) = child {
                children.push((chars, child, start));
            }
        }
        for (chars, child, start) in children {
            if child != self.r.pos {
                return Err(format!(
                    "node {start}'s children are not where depth-first order puts them"
                ));
            }
            let length = prefix.len();
            prefix.extend_from_slice(&chars);
            self.array(prefix, depth + 1)?;
            prefix.truncate(length);
        }
        Ok(())
    }

    /// The node at the reader's position: its characters and its children's position.
    fn node(&mut self, prefix: &[u32]) -> Result<(Vec<u32>, Option<usize>), String> {
        let r = &mut self.r;
        let pos = r.pos;
        let flags = r.u8()?;
        let chars = if flags & MULTIPLE_CHARS != 0 {
            r.string()?
        } else {
            vec![r.code_point()?.ok_or("a node with no character")?]
        };
        let probability = if flags & TERMINAL != 0 {
            Some(r.u8()?)
        } else {
            None
        };
        let children = match (flags & CHILDREN_ADDRESS_MASK) >> 6 {
            0 => None,
            size => {
                let field = r.pos;
                Some(field + r.uint(usize::from(size))? as usize)
            }
        };
        let mut shortcuts = Vec::new();
        if flags & HAS_SHORTCUTS != 0 {
            let end = r.pos + r.uint(2)? as usize;
            loop {
                let entry = r.u8()?;
                shortcuts.push((r.string()?, entry & PROBABILITY_MASK));
                if entry & ATTRIBUTE_HAS_NEXT == 0 {
                    break;
                }
            }
            if r.pos != end {
                return Err(format!(
                    "node {pos}'s shortcut list is not the size it says"
                ));
            }
        }
        let mut bigrams = Vec::new();
        if flags & HAS_BIGRAMS != 0 {
            loop {
                let entry = r.u8()?;
                let field = r.pos as i64;
                let size = usize::from((entry & BIGRAM_ADDRESS_MASK) >> 4);
                let offset = i64::from(r.uint(size)?);
                let target = if entry & BIGRAM_OFFSET_NEGATIVE != 0 {
                    field - offset
                } else {
                    field + offset
                };
                bigrams.push((
                    usize::try_from(target).map_err(|e| e.to_string())?,
                    entry & PROBABILITY_MASK,
                ));
                if entry & ATTRIBUTE_HAS_NEXT == 0 {
                    break;
                }
            }
        }
        let entry = match probability {
            Some(probability) => {
                let mut word = prefix.to_vec();
                word.extend_from_slice(&chars);
                self.entries.push(Entry {
                    word,
                    probability,
                    not_a_word: flags & NOT_A_WORD != 0,
                    possibly_offensive: flags & POSSIBLY_OFFENSIVE != 0,
                    shortcuts,
                    bigrams: Vec::new(),
                });
                Some(self.entries.len() - 1)
            }
            None if shortcuts.is_empty() && bigrams.is_empty() => None,
            None => return Err(format!("node {pos} has attributes but is no word")),
        };
        self.nodes.insert(pos, ReadNode { entry, bigrams });
        Ok((chars, children))
    }
}
