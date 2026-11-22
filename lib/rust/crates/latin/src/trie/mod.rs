//! The version 2 trie: the dictionary structure `dictionary/structure/v2`, its shared reading code
//! `dictionary/structure/pt_common`, the dictionary-level word and probability queries of
//! `suggest/core/dictionary/dictionary`, and the JNI glue of `com_android_inputmethod_latin_BinaryDictionary`.

pub mod bigram;
pub mod code_points;
pub mod dictionary;
pub mod dynamic;
pub mod factory;
pub mod interface;
pub mod jni;
pub mod multi_bigram;
pub mod ngram;
pub mod node;
pub mod patricia;
pub mod probability;
pub mod reading;
pub mod shortcut;
pub mod words;

#[cfg(test)]
mod tests;
