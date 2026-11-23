//! The clipboard store's payload, kind `CLIP`, version 1:
//!
//! | What | How |
//! |---|---|
//! | next id | `i64` |
//! | imported databases | count, then each fingerprint as a `u64` |
//! | clips | count, then each: id `i64`, type `i32`, time `i64`, flags `u8` (1 pinned, 2 sensitive, 4 from another device), text and URI each optional, MIME types text |
//! | media files | count, then each: id `i64`, size `i64`, orientation `i32`, display name text, MIME types text |

use crate::clips::{Clip, Clips, FileInfo};
use crate::codec::{Error, Reader, Writer};
use crate::store::Payload;

const PINNED: u8 = 1;
const SENSITIVE: u8 = 2;
const REMOTE: u8 = 4;

/// The fewest bytes a clip takes: its numbers, flags, two absent texts and an empty MIME text.
const CLIP_BYTES: usize = 8 + 4 + 8 + 1 + 1 + 1 + 4;
/// The fewest bytes a media file's record takes.
const FILE_BYTES: usize = 8 + 8 + 4 + 4 + 4;

impl Payload for Clips {
    const KIND: [u8; 4] = *b"CLIP";
    const VERSION: u32 = 1;

    fn encode(&self, writer: &mut Writer) {
        writer.i64(self.next_id);
        writer.count(self.imported.len());
        self.imported.iter().for_each(|&print| writer.u64(print));
        writer.count(self.items.len());
        self.items.iter().for_each(|clip| write_clip(writer, clip));
        writer.count(self.files.len());
        self.files.iter().for_each(|file| write_file(writer, file));
    }

    fn decode(reader: &mut Reader<'_>, _version: u32) -> Result<Self, Error> {
        let next_id = reader.i64()?;
        let imported = (0..reader.count(8)?)
            .map(|_| reader.u64())
            .collect::<Result<_, _>>()?;
        let items: Vec<Clip> = (0..reader.count(CLIP_BYTES)?)
            .map(|_| read_clip(reader))
            .collect::<Result<_, _>>()?;
        let files = (0..reader.count(FILE_BYTES)?)
            .map(|_| read_file(reader))
            .collect::<Result<_, _>>()?;
        let ordered = items.windows(2).all(|pair| pair[0].id < pair[1].id);
        if !ordered || items.last().is_some_and(|clip| clip.id >= next_id) {
            return Err(Error::Damaged("clip ids"));
        }
        Ok(Clips {
            next_id,
            imported,
            items,
            files,
        })
    }
}

pub(crate) fn write_clip(writer: &mut Writer, clip: &Clip) {
    writer.i64(clip.id);
    writer.i32(clip.kind);
    writer.i64(clip.created);
    let flag = |on: bool, bit: u8| if on { bit } else { 0 };
    writer.u8(flag(clip.pinned, PINNED)
        | flag(clip.sensitive, SENSITIVE)
        | flag(clip.remote, REMOTE));
    writer.optional_text(clip.text.as_deref());
    writer.optional_text(clip.uri.as_deref());
    writer.text(&clip.mime_types);
}

fn read_clip(reader: &mut Reader<'_>) -> Result<Clip, Error> {
    let id = reader.i64()?;
    let kind = reader.i32()?;
    let created = reader.i64()?;
    let flags = reader.u8()?;
    if flags & !(PINNED | SENSITIVE | REMOTE) != 0 {
        return Err(Error::Damaged("clip flags"));
    }
    Ok(Clip {
        id,
        kind,
        created,
        pinned: flags & PINNED != 0,
        sensitive: flags & SENSITIVE != 0,
        remote: flags & REMOTE != 0,
        text: reader.optional_text()?,
        uri: reader.optional_text()?,
        mime_types: reader.text()?,
    })
}

pub(crate) fn write_file(writer: &mut Writer, file: &FileInfo) {
    writer.i64(file.id);
    writer.i64(file.size);
    writer.i32(file.orientation);
    writer.text(&file.name);
    writer.text(&file.mime_types);
}

fn read_file(reader: &mut Reader<'_>) -> Result<FileInfo, Error> {
    Ok(FileInfo {
        id: reader.i64()?,
        size: reader.i64()?,
        orientation: reader.i32()?,
        name: reader.text()?,
        mime_types: reader.text()?,
    })
}
