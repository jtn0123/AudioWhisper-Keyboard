//! How a boolean crosses in a string packet. Both sides encode and decode through these, never
//! through a literal "1" or "0".

crate::declare! {
    /// A boolean as it crosses in a string packet.
    pub const FLAG: flag WireFlag {
        /// True.
        TRUE = "1",
        /// False.
        FALSE = "0",
    }
}

/// `value` as it crosses.
pub const fn encode(value: bool) -> &'static str {
    if value { TRUE } else { FALSE }
}

/// `value` as it crosses, in UTF-16.
pub fn utf16(value: bool) -> Vec<u16> {
    encode(value).encode_utf16().collect()
}

/// Whether `units` is [`TRUE`]. Anything else, [`FALSE`] included, is false.
pub fn decode(units: &[u16]) -> bool {
    units.iter().copied().eq(TRUE.encode_utf16())
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[&FLAG];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_boolean_crosses_and_comes_back() {
        for value in [false, true] {
            assert_eq!(decode(&utf16(value)), value);
        }
        assert!(!decode(&utf16(false)) && !decode(&[]) && !decode(&[u16::from(b'2')]));
        assert_eq!((encode(true), encode(false)), ("1", "0"));
    }
}
