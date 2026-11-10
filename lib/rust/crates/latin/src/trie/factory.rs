//! `dictionary/structure/dictionary_structure_with_buffer_policy_factory`: picks the structure policy
//! for a dictionary file by its format version. Only version 202 (the version 2 patricia trie) is
//! ported; the other versions the C++ handles (2, 201, 4 and later) are not.

use std::sync::Arc;

use super::patricia::PatriciaTriePolicy;
use crate::bytes::{FormatVersion, detect_format_version};
use crate::header::HeaderPolicy;

/// `DictionaryStructureWithBufferPolicyFactory::newPolicyForFileDict`, for the versions ported: the
/// file's header and its structure policy, which starts after the header and reads the header's
/// code point table, as `PatriciaTriePolicy(mmappedBuffer)` does. `None` for a file of any other
/// version, or one too short to have a header.
pub fn new_policy_for_file(file: Arc<[u8]>) -> Option<(HeaderPolicy, PatriciaTriePolicy)> {
    let version = detect_format_version(&file);
    match version {
        FormatVersion::Version202 => {
            let header = HeaderPolicy::new(&file, version);
            let code_point_table = header.code_point_table().map(<[i32]>::to_vec);
            let policy =
                PatriciaTriePolicy::new(Arc::clone(&file), header.size(), code_point_table);
            Some((header, policy))
        }
        _ => None,
    }
}
