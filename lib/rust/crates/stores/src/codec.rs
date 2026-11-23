//! The file format every store shares, little-endian throughout:
//!
//! | Bytes | What |
//! |---|---|
//! | 4 | `AWST`, the magic |
//! | 4 | the store's kind, such as `CLIP` or `WORD` |
//! | 4 | the payload's version, a `u32` |
//! | 8 | the payload's length in bytes, a `u64` |
//! | n | the payload |
//! | 4 | CRC-32 (IEEE) of every byte before it |
//!
//! A payload is built from the pieces [`Writer`] writes and [`Reader`] reads back: integers of a
//! fixed width, and text as a `u32` count of UTF-16 units followed by the units, so a lone
//! surrogate survives. An absent text is a `0` byte; a present one a `1` byte and the text.

use std::fmt;

/// The first four bytes of every store.
pub const MAGIC: [u8; 4] = *b"AWST";

/// How many bytes come before the payload.
const HEADER: usize = 20;

/// Why a file is not a readable store of the kind asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Not a store, or not of this kind.
    Kind,
    /// A version newer than this build reads.
    Newer(u32),
    /// Cut short, too long, or its checksum does not match.
    Damaged(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Kind => f.write_str("not a store of this kind"),
            Error::Newer(version) => write!(f, "written by a newer version ({version})"),
            Error::Damaged(why) => write!(f, "damaged: {why}"),
        }
    }
}

/// `payload` framed as a store of `kind` at `version`.
pub fn frame(kind: [u8; 4], version: u32, payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(HEADER + payload.len() + 4);
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&kind);
    bytes.extend_from_slice(&version.to_le_bytes());
    bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes.extend_from_slice(payload);
    let crc = crc32(&bytes);
    bytes.extend_from_slice(&crc.to_le_bytes());
    bytes
}

/// The version and payload of a store of `kind`, refusing a version above `newest`.
pub fn unframe(kind: [u8; 4], newest: u32, bytes: &[u8]) -> Result<(u32, &[u8]), Error> {
    if bytes.len() < 8 || bytes[..4] != MAGIC || bytes[4..8] != kind {
        return Err(Error::Kind);
    }
    if bytes.len() < HEADER + 4 {
        return Err(Error::Damaged("cut short"));
    }
    let (body, crc) = bytes.split_at(bytes.len() - 4);
    if crc32(body).to_le_bytes() != crc {
        return Err(Error::Damaged("checksum"));
    }
    let version = u32::from_le_bytes(body[8..12].try_into().expect("four bytes"));
    let length = u64::from_le_bytes(body[12..HEADER].try_into().expect("eight bytes"));
    if length != (body.len() - HEADER) as u64 {
        return Err(Error::Damaged("length"));
    }
    if version > newest {
        return Err(Error::Newer(version));
    }
    Ok((version, &body[HEADER..]))
}

/// CRC-32 with the IEEE polynomial, as zlib computes it.
pub fn crc32(bytes: &[u8]) -> u32 {
    const TABLE: [u32; 256] = crc_table();
    let mut crc = !0u32;
    for &byte in bytes {
        crc = TABLE[((crc ^ u32::from(byte)) & 0xff) as usize] ^ (crc >> 8);
    }
    !crc
}

const fn crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut n = 0;
    while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 {
                0xedb8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
            k += 1;
        }
        table[n] = c;
        n += 1;
    }
    table
}

/// FNV-1a, 64 bits: a fingerprint of bytes, such as the old database rows [`crate::legacy`]
/// imports, to know them again.
pub fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// Builds a payload.
#[derive(Debug, Default)]
pub struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn i64(&mut self, value: i64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    /// A count of what follows; a store never holds more than `u32::MAX` of anything.
    pub fn count(&mut self, count: usize) {
        self.u32(u32::try_from(count).expect("fewer than 2^32"));
    }

    pub fn text(&mut self, units: &[u16]) {
        self.count(units.len());
        for unit in units {
            self.bytes.extend_from_slice(&unit.to_le_bytes());
        }
    }

    pub fn optional_text(&mut self, units: Option<&[u16]>) {
        match units {
            None => self.u8(0),
            Some(units) => {
                self.u8(1);
                self.text(units);
            }
        }
    }
}

/// Reads a payload back, refusing anything cut short.
#[derive(Debug)]
pub struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn new(payload: &'a [u8]) -> Self {
        Reader { rest: payload }
    }

    fn take<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        if self.rest.len() < N {
            return Err(Error::Damaged("cut short"));
        }
        let (head, rest) = self.rest.split_at(N);
        self.rest = rest;
        Ok(head.try_into().expect("N bytes"))
    }

    pub fn u8(&mut self) -> Result<u8, Error> {
        Ok(self.take::<1>()?[0])
    }

    pub fn u32(&mut self) -> Result<u32, Error> {
        self.take().map(u32::from_le_bytes)
    }

    pub fn i32(&mut self) -> Result<i32, Error> {
        self.take().map(i32::from_le_bytes)
    }

    pub fn i64(&mut self) -> Result<i64, Error> {
        self.take().map(i64::from_le_bytes)
    }

    pub fn u64(&mut self) -> Result<u64, Error> {
        self.take().map(u64::from_le_bytes)
    }

    /// A count, which can't be more than the bytes left of items at least `size` bytes each.
    pub fn count(&mut self, size: usize) -> Result<usize, Error> {
        let count = self.u32()? as usize;
        if count.saturating_mul(size) > self.rest.len() {
            return Err(Error::Damaged("count"));
        }
        Ok(count)
    }

    pub fn text(&mut self) -> Result<Vec<u16>, Error> {
        let count = self.count(2)?;
        let (units, rest) = self.rest.split_at(count * 2);
        self.rest = rest;
        Ok(units
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&pair| u16::from_le_bytes(pair))
            .collect())
    }

    pub fn optional_text(&mut self) -> Result<Option<Vec<u16>>, Error> {
        match self.u8()? {
            0 => Ok(None),
            1 => self.text().map(Some),
            _ => Err(Error::Damaged("optional text")),
        }
    }

    /// Checks that nothing is left over.
    pub fn finish(self) -> Result<(), Error> {
        if self.rest.is_empty() {
            Ok(())
        } else {
            Err(Error::Damaged("trailing bytes"))
        }
    }
}
