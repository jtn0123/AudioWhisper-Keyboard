//! Decode the mono PCM16 Loona reference without Android or a sherpa JNI dependency.
pub fn pcm16(bytes: &[u8]) -> Result<(Vec<f32>, i32), &'static str> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("Invalid WAV header");
    }
    let mut at = 12usize;
    let mut rate = None;
    let mut data = None;
    while at.checked_add(8).is_some_and(|end| end <= bytes.len()) {
        let size =
            u32::from_le_bytes(bytes[at + 4..at + 8].try_into().map_err(|_| "WAV size")?) as usize;
        let start = at + 8;
        let end = start.checked_add(size).ok_or("WAV overflow")?;
        let chunk = bytes.get(start..end).ok_or("Truncated WAV")?;
        match &bytes[at..at + 4] {
            b"fmt " => {
                if chunk.len() < 16 || chunk[0..4] != [1, 0, 1, 0] || chunk[14..16] != [16, 0] {
                    return Err("Expected mono PCM16");
                }
                let value = i32::from_le_bytes(chunk[4..8].try_into().map_err(|_| "WAV rate")?);
                if value <= 0 {
                    return Err("Invalid WAV rate");
                }
                rate = Some(value);
            }
            b"data" => data = Some(chunk),
            _ => {}
        }
        at = end.checked_add(size % 2).ok_or("WAV overflow")?;
    }
    let data = data.ok_or("Missing WAV PCM")?;
    if !data.len().is_multiple_of(2) {
        return Err("Partial WAV sample");
    }
    Ok((
        data.as_chunks::<2>()
            .0
            .iter()
            .map(|b| f32::from(i16::from_le_bytes([b[0], b[1]])) / 32768.0)
            .collect(),
        rate.ok_or("Missing WAV format")?,
    ))
}

#[cfg(test)]
mod tests;
