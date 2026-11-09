//! `ByteArrayUtils` and `FormatUtils`, from `cpp/hooks/bytes.cpp`. The C++ reads without bounds
//! checks, so each call copies the buffer with zeros after it, past anywhere the call can reach;
//! the Rust port reads a byte outside its buffer as 0, so the two see the same bytes.

/// Enough zeros for a string of 255 three-byte code points and its terminator.
const PADDING: usize = 4 * 256;

// SAFETY: cpp/hooks/bytes.cpp defines these with the same C signatures.
unsafe extern "C" {
    fn aw_hook_read_uint(buffer: *const u8, size: i32, pos: i32) -> u32;
    fn aw_hook_read_s24_advance(buffer: *const u8, pos: *mut i32) -> i32;
    fn aw_hook_read_code_point_advance(buffer: *const u8, table: *const i32, pos: *mut i32) -> i32;
    fn aw_hook_read_string_advance(
        buffer: *const u8,
        max_length: i32,
        table: *const i32,
        out: *mut i32,
        pos: *mut i32,
    ) -> i32;
    fn aw_hook_advance_past_string(buffer: *const u8, max_length: i32, pos: *mut i32) -> i32;
    fn aw_hook_detect_format_version(buffer: *const u8, size: usize) -> i32;
}

fn padded(buffer: &[u8]) -> Vec<u8> {
    let mut copy = buffer.to_vec();
    copy.resize(buffer.len() + PADDING, 0);
    copy
}

/// A position the padded copy holds a call's reads from.
fn check(buffer: &[u8], pos: i32) {
    assert!(
        pos >= 0 && pos as usize <= buffer.len(),
        "position {pos} is outside the buffer"
    );
}

/// The code point table as the C++ takes it: 224 entries (one per one-byte value), or none.
fn table_pointer(table: Option<&[i32]>) -> *const i32 {
    match table {
        Some(table) => {
            assert_eq!(
                table.len(),
                224,
                "the C++ indexes the table by any byte from 0x20"
            );
            table.as_ptr()
        }
        None => std::ptr::null(),
    }
}

/// `readUint`.
pub fn read_uint(buffer: &[u8], size: i32, pos: i32) -> u32 {
    check(buffer, pos);
    let copy = padded(buffer);
    // SAFETY: pos is in the buffer and at most 4 bytes are read past it, inside the padding.
    unsafe { aw_hook_read_uint(copy.as_ptr(), size, pos) }
}

/// `readSint24AndAdvancePosition`.
pub fn read_s24_advance(buffer: &[u8], pos: &mut i32) -> i32 {
    check(buffer, *pos);
    let copy = padded(buffer);
    // SAFETY: as read_uint; pos is a valid int to write.
    unsafe { aw_hook_read_s24_advance(copy.as_ptr(), pos) }
}

/// `readCodePointAndAdvancePosition`.
pub fn read_code_point_advance(buffer: &[u8], table: Option<&[i32]>, pos: &mut i32) -> i32 {
    check(buffer, *pos);
    let copy = padded(buffer);
    let table = table_pointer(table);
    // SAFETY: as read_uint; the table, if any, has an entry for every one-byte value.
    unsafe { aw_hook_read_code_point_advance(copy.as_ptr(), table, pos) }
}

/// `readStringAndAdvancePosition`; returns the code points read.
pub fn read_string_advance(
    buffer: &[u8],
    max_length: usize,
    table: Option<&[i32]>,
    pos: &mut i32,
) -> Vec<i32> {
    check(buffer, *pos);
    assert!(max_length < 255, "the padding holds 255 code points");
    let copy = padded(buffer);
    let table = table_pointer(table);
    let mut out = vec![0; max_length];
    // SAFETY: at most max_length + 1 code points of 3 bytes are read, inside the padding; out
    // holds max_length.
    let length = unsafe {
        aw_hook_read_string_advance(
            copy.as_ptr(),
            max_length as i32,
            table,
            out.as_mut_ptr(),
            pos,
        )
    };
    out.truncate(length as usize);
    out
}

/// `advancePositionToBehindString`.
pub fn advance_past_string(buffer: &[u8], max_length: usize, pos: &mut i32) -> i32 {
    check(buffer, *pos);
    assert!(max_length < 255, "the padding holds 255 code points");
    let copy = padded(buffer);
    // SAFETY: as read_string_advance.
    unsafe { aw_hook_advance_past_string(copy.as_ptr(), max_length as i32, pos) }
}

/// `FormatUtils::detectFormatVersion`, as its enum's value.
pub fn detect_format_version(buffer: &[u8]) -> i32 {
    // SAFETY: the C++ reads at most the 12 bytes it checks the size for.
    unsafe { aw_hook_detect_format_version(buffer.as_ptr(), buffer.len()) }
}
