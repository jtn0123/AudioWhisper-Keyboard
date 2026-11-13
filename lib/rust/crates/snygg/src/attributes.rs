//! Attribute maps preserve insertion order, but comparison and formatting sort their keys.
use crate::{Text, compare, text};

#[derive(Clone, Debug, Default, Eq)]
pub struct Attributes(pub Vec<(Text, Vec<Text>)>);

impl Attributes {
    pub fn compare(&self, other: &Self) -> i32 {
        let size = self.0.len().cmp(&other.0.len());
        if !size.is_eq() {
            return if size.is_lt() { -1 } else { 1 };
        }
        let mut a: Vec<_> = self.0.iter().collect();
        let mut b: Vec<_> = other.0.iter().collect();
        a.sort_by_key(|entry| &entry.0);
        b.sort_by_key(|entry| &entry.0);
        for ((key, values), (other_key, other_values)) in a.into_iter().zip(b) {
            let diff = compare(key, other_key);
            if diff != 0 {
                return diff;
            }
            for (value, other_value) in values.iter().zip(other_values) {
                let diff = compare(value, other_value);
                if diff != 0 {
                    return diff;
                }
            }
            match values.len().cmp(&other_values.len()) {
                std::cmp::Ordering::Less => return -1,
                std::cmp::Ordering::Greater => return 1,
                std::cmp::Ordering::Equal => {}
            }
        }
        0
    }

    pub fn format(&self) -> Text {
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by_key(|entry| &entry.0);
        let mut out = Vec::new();
        for (key, values) in entries {
            if values.is_empty() {
                continue;
            }
            out.push(u16::from(b'['));
            out.extend(key);
            out.push(u16::from(b'='));
            let (mut ints, strings): (Vec<_>, Vec<_>) = values
                .iter()
                .partition(|v| chars::text::to_int_or_null(v).is_some());
            // Deliberately lexical, as in Kotlin: 1,10,2 is not a numeric sort.
            ints.sort();
            let mut ranges: Vec<(i32, i32)> = Vec::new();
            for value in ints {
                if let Some(value) = chars::text::to_int_or_null(value) {
                    if let Some(last) = ranges.last_mut()
                        && last.1.wrapping_add(1) == value
                    {
                        last.1 = value;
                        continue;
                    }
                    ranges.push((value, value));
                }
            }
            let mut serialized = Vec::new();
            for (start, end) in ranges {
                serialized.push(text(&start.to_string()));
                if start == end.wrapping_sub(1) {
                    serialized.push(text(&end.to_string()));
                } else if start != end {
                    let last = serialized.len() - 1;
                    serialized[last].extend(text(&format!("..{end}")));
                }
            }
            let mut strings: Vec<_> = strings
                .iter()
                .map(|s| [text("`"), s.to_vec(), text("`")].concat())
                .collect();
            strings.sort();
            serialized.extend(strings);
            for (i, value) in serialized.into_iter().enumerate() {
                if i != 0 {
                    out.push(u16::from(b','));
                }
                out.extend(value);
            }
            out.push(u16::from(b']'));
        }
        out
    }

    /// 0 includes, 1 excludes, 2 toggles. Each pair observes the preceding pair's edits.
    pub fn edit(&mut self, pairs: &[(Text, Text)], mode: u8) {
        for (key, value) in pairs {
            let index = self.0.iter().position(|(k, _)| k == key);
            let mut values = index.map(|i| self.0[i].1.clone()).unwrap_or_default();
            let had = values.contains(value);
            if mode == 1 || (mode == 2 && had) {
                values.retain(|v| v != value);
            } else if !had {
                values.push(value.clone());
            }
            values.sort();
            values.dedup();
            match index {
                Some(i) if values.is_empty() => {
                    self.0.remove(i);
                }
                Some(i) => self.0[i].1 = values,
                None if !values.is_empty() => self.0.push((key.clone(), values)),
                None => {}
            }
        }
    }

    pub fn put(&mut self, key: Text, values: Vec<Text>) {
        if let Some((_, old)) = self.0.iter_mut().find(|(k, _)| *k == key) {
            *old = values;
        } else {
            self.0.push((key, values));
        }
    }
}

impl PartialEq for Attributes {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self
                .0
                .iter()
                .all(|(k, v)| other.0.iter().any(|(ok, ov)| k == ok && v == ov))
    }
}
