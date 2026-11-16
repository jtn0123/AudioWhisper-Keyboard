//! Writing [`Tables`] as a model file (`ngram::format`).

use super::smooth::Tables;
use ngram::format::{HEADER_LEN, Header, Layout, MAX_WORDS};

fn push(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn pad(out: &mut Vec<u8>) {
    out.resize(out.len().next_multiple_of(4), 0);
}

fn count(n: usize, what: &str) -> Result<u32, String> {
    u32::try_from(n).map_err(|_| format!("too many {what} for the model file"))
}

pub fn write(tables: &Tables) -> Result<Vec<u8>, String> {
    let words = count(tables.words.len(), "words")?;
    if words >= MAX_WORDS {
        return Err("too many words for the model file".to_string());
    }
    let units: usize = tables.words.iter().map(Vec::len).sum();
    let header = Header {
        words,
        units: count(units, "UTF-16 units")?,
        bigrams: count(tables.bigrams.len(), "bigrams")?,
        contexts: count(tables.contexts.len(), "contexts")?,
        trigrams: count(tables.trigrams.len(), "trigrams")?,
        step: tables.step,
        backoff_step: tables.backoff_step,
        scale_a: tables.scale.a as f32,
        scale_b: tables.scale.b as f32,
    };
    let sizes = [
        (tables.unigrams.len(), tables.words.len() + 1, "unigrams"),
        (
            tables.bigram_starts.len(),
            tables.words.len() + 2,
            "bigram starts",
        ),
        (tables.backoffs.len(), tables.bigrams.len(), "backoffs"),
        (
            tables.trigram_starts.len(),
            tables.contexts.len() + 1,
            "trigram starts",
        ),
    ];
    for (got, want, what) in sizes {
        if got != want {
            return Err(format!("{got} {what}, expected {want}"));
        }
    }
    let layout = Layout::of(&header).ok_or("the model file would be too large")?;
    let mut out = Vec::with_capacity(layout.end);
    out.extend_from_slice(&header.to_bytes());
    debug_assert_eq!(out.len(), HEADER_LEN);
    let mut at = 0;
    push(&mut out, 0);
    for word in &tables.words {
        at += word.len() as u32;
        push(&mut out, at);
    }
    for word in &tables.words {
        for unit in word {
            out.extend_from_slice(&unit.to_le_bytes());
        }
    }
    pad(&mut out);
    for unigram in &tables.unigrams {
        out.extend_from_slice(&unigram.to_bytes());
    }
    for &v in tables.bigram_starts.iter().chain(&tables.bigrams) {
        push(&mut out, v);
    }
    out.extend_from_slice(&tables.backoffs);
    pad(&mut out);
    let rest = tables.contexts.iter().chain(&tables.trigram_starts);
    for &v in rest.chain(&tables.trigrams) {
        push(&mut out, v);
    }
    if out.len() != layout.end {
        return Err(format!(
            "wrote {} bytes, expected {}",
            out.len(),
            layout.end
        ));
    }
    Ok(out)
}
