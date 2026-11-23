//! The microphone's audio rules: how loud a chunk is, what a read's count means, the PCM in a
//! WAV file played in place of the mic, and how large the mic's buffer is. The keyboard's
//! `MicRecorder` and `MicAudio` did this in Kotlin; these give exactly the same values (the
//! keyboard's `MicTraceTest` compares them with main's code).

use wire::dictation_edges::read;

use crate::segmentation::SAMPLE_RATE;

/// How loud `samples` are, 0 to 1: their root mean square times 8, so ordinary speech fills most
/// of the meter. Squares are taken in `f32` and summed in `f64`, as Kotlin did.
pub fn level(samples: &[f32]) -> f32 {
    let sum: f64 = samples.iter().map(|&s| f64::from(s * s)).sum();
    let rms = (sum / samples.len() as f64).sqrt() * 8.0;
    (rms as f32).clamp(0.0, 1.0)
}

/// What a read that returned `count` means for the recording thread, while it is still `running`
/// or not: a `MicRead` code. An error ends recording, and is reported only if nobody stopped it.
pub fn read(count: i32, running: bool) -> i32 {
    match count {
        ..0 if running => read::REPORT,
        ..0 => read::END,
        0 => read::SKIP,
        _ => read::DELIVER,
    }
}

/// The bytes the mic's `AudioRecord` buffers: at least half a second of float samples, or
/// `minimum`, Android's own floor, if that is more.
pub fn buffer_bytes(minimum: i32) -> i32 {
    const HALF_SECOND: i32 = (SAMPLE_RATE * size_of::<f32>() / 2) as i32;
    minimum.max(HALF_SECOND)
}

/// Why a WAV file has no audio to play.
#[derive(Debug, PartialEq, Eq)]
pub enum WavError {
    /// It has no RIFF/WAVE header.
    NotWav,
    /// No `data` chunk, or a chunk size the walk can't follow (see [`pcm16`]).
    NoAudio,
}

impl WavError {
    /// The message Kotlin threw.
    pub fn message(&self) -> &'static str {
        match self {
            WavError::NotWav => "Not a WAV file",
            WavError::NoAudio => "No audio in the WAV file",
        }
    }
}

/// The samples of a 16 kHz mono 16-bit PCM WAV file, each `i16` over 32,768: the first `data`
/// chunk the walk reaches, cut to the bytes the file holds. Chunks are padded to an even size.
///
/// The walk is Kotlin's, in its 32-bit arithmetic, so a chunk size that moves it back is followed
/// as it was. Where Kotlin never returned (the walk came back to a chunk it had read, which takes
/// more steps than the file has bytes) or threw something other than IllegalArgumentException (a
/// chunk before the start of the file, or a data chunk of a negative sample count), this answers
/// [`WavError::NoAudio`].
pub fn pcm16(wav: &[u8]) -> Result<Vec<f32>, WavError> {
    if wav.len() <= 12 || &wav[0..4] != b"RIFF" || &wav[8..12] != b"WAVE" {
        return Err(WavError::NotWav);
    }
    let length = wav.len() as i64;
    let mut offset: i32 = 12;
    for _ in 0..=wav.len() {
        if i64::from(offset) + 8 > length {
            return Err(WavError::NoAudio);
        }
        let at = usize::try_from(offset).map_err(|_| WavError::NoAudio)?;
        let size = i32::from_le_bytes([wav[at + 4], wav[at + 5], wav[at + 6], wav[at + 7]]);
        if &wav[at..at + 4] == b"data" {
            let available = (length - i64::from(offset) - 8) as i32;
            let count = usize::try_from(size.min(available) / 2).map_err(|_| WavError::NoAudio)?;
            let (pairs, _) = wav[at + 8..at + 8 + count * 2].as_chunks::<2>();
            return Ok(pairs
                .iter()
                .map(|&pair| f32::from(i16::from_le_bytes(pair)) / 32768.0)
                .collect());
        }
        offset = offset.wrapping_add(8i32.wrapping_add(size).wrapping_add(size & 1));
    }
    Err(WavError::NoAudio)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav(chunks: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
        let mut out = b"RIFF\0\0\0\0WAVE".to_vec();
        for (id, body) in chunks {
            out.extend_from_slice(*id);
            out.extend_from_slice(&(body.len() as i32).to_le_bytes());
            out.extend_from_slice(body);
            if body.len() % 2 == 1 {
                out.push(0);
            }
        }
        out
    }

    #[test]
    fn levels_are_rms_times_eight_capped_at_one() {
        assert_eq!(level(&[0.0; 4]), 0.0);
        assert_eq!(
            level(&[0.05, -0.05]),
            (f64::from(0.05f32 * 0.05).sqrt() * 8.0) as f32
        );
        assert_eq!(level(&[0.5; 3]), 1.0);
    }

    #[test]
    fn reads_end_or_report_errors_and_skip_nothing() {
        assert_eq!(read(-3, true), read::REPORT);
        assert_eq!(read(-3, false), read::END);
        assert_eq!(read(0, true), read::SKIP);
        assert_eq!(read(0, false), read::SKIP);
        assert_eq!(read(1600, false), read::DELIVER);
    }

    #[test]
    fn the_buffer_holds_half_a_second_at_least() {
        assert_eq!(buffer_bytes(-2), 32_000);
        assert_eq!(buffer_bytes(40_000), 40_000);
    }

    #[test]
    fn wav_audio_is_the_first_data_chunk_after_padded_others() {
        let samples: Vec<u8> = [i16::MIN, -1, 0, 16384, i16::MAX]
            .iter()
            .flat_map(|s| s.to_le_bytes())
            .collect();
        let file = wav(&[
            (b"fmt ", &[1, 2, 3]),
            (b"data", &samples),
            (b"data", &[9, 9]),
        ]);
        assert_eq!(
            pcm16(&file),
            Ok(vec![-1.0, -1.0 / 32768.0, 0.0, 0.5, 32767.0 / 32768.0])
        );
    }

    #[test]
    fn a_size_that_moves_back_to_a_new_chunk_is_followed() {
        let mut file = wav(&[(b"aaaa", &[0; 20]), (b"bbbb", &[])]);
        file[22..26].copy_from_slice(b"data");
        file[26..30].copy_from_slice(&2i32.to_le_bytes());
        file[30..32].copy_from_slice(&[0, 64]);
        file[44..48].copy_from_slice(&(-26i32).to_le_bytes());
        assert_eq!(pcm16(&file), Ok(vec![0.5]));
        let mut before_the_start = wav(&[(b"aaaa", &[])]);
        before_the_start[16..20].copy_from_slice(&(-30i32).to_le_bytes());
        assert_eq!(pcm16(&before_the_start), Err(WavError::NoAudio));
    }

    #[test]
    fn a_negative_size_that_moves_on_is_followed() {
        let mut file = wav(&[(b"junk", &[]), (b"data", &[0, 64])]);
        file[16..20].copy_from_slice(&(-1i32).to_le_bytes());
        assert_eq!(pcm16(&file), Ok(vec![0.5]));
        let mut empty = wav(&[(b"data", &[0, 64])]);
        empty[16..20].copy_from_slice(&(-1i32).to_le_bytes());
        assert_eq!(pcm16(&empty), Ok(vec![]));
    }

    #[test]
    fn a_cut_short_data_chunk_keeps_its_whole_samples() {
        let mut file = wav(&[(b"data", &[0, 64, 0, 64])]);
        file.truncate(file.len() - 1);
        assert_eq!(pcm16(&file), Ok(vec![0.5]));
    }

    #[test]
    fn files_without_audio_are_refused() {
        assert_eq!(pcm16(b"RIFF\0\0\0\0WAVE"), Err(WavError::NotWav));
        assert_eq!(pcm16(b"RIFX\0\0\0\0WAVEdata"), Err(WavError::NotWav));
        assert_eq!(pcm16(&wav(&[(b"fmt ", &[1, 2])])), Err(WavError::NoAudio));
        let mut stuck = wav(&[(b"fmt ", &[1, 2])]);
        stuck[16..20].copy_from_slice(&(-8i32).to_le_bytes());
        assert_eq!(pcm16(&stuck), Err(WavError::NoAudio));
        let mut negative = wav(&[(b"data", &[1, 2, 3, 4])]);
        negative[16..20].copy_from_slice(&(-2i32).to_le_bytes());
        assert_eq!(pcm16(&negative), Err(WavError::NoAudio));
        let mut huge = wav(&[(b"fmt ", &[1, 2])]);
        huge[16..20].copy_from_slice(&i32::MAX.to_le_bytes());
        assert_eq!(pcm16(&huge), Err(WavError::NoAudio));
        assert_eq!(WavError::NoAudio.message(), "No audio in the WAV file");
    }
}
