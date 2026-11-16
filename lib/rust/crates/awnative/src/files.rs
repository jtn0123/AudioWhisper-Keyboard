//! Reading the app's bundled files.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

/// The bytes `[offset, offset + size)` of the file at `path`, which may be an uncompressed entry
/// inside an APK. A range past the file's end is an error, found before anything is allocated.
pub fn read_range(path: &str, offset: u64, size: usize) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    let end = u64::try_from(size)
        .ok()
        .and_then(|size| offset.checked_add(size));
    if end.is_none_or(|end| end > len) {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = vec![0; size];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}
