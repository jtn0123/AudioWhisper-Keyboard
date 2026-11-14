use super::*;
#[test]
fn pcm16_extremes_match_sherpa_scaling_and_skip_unknown_chunks() {
    let mut bytes = b"RIFF\0\0\0\0WAVEJUNK\x01\0\0\0x\0fmt \x10\0\0\0\x01\0\x01\0".to_vec();
    bytes.extend(24000i32.to_le_bytes());
    bytes.extend(48000i32.to_le_bytes());
    bytes.extend([2, 0, 16, 0]);
    bytes.extend(b"data\x06\0\0\0");
    bytes.extend(i16::MIN.to_le_bytes());
    bytes.extend(0i16.to_le_bytes());
    bytes.extend(i16::MAX.to_le_bytes());
    let (samples, rate) = pcm16(&bytes).unwrap();
    assert_eq!(rate, 24000);
    assert_eq!(samples, [-1.0, 0.0, 32767.0 / 32768.0]);
    bytes.pop();
    assert!(pcm16(&bytes).is_err());
}
#[test]
fn invalid_input_is_an_error() {
    for bytes in [&b""[..], &b"RIFF\0\0\0\0WAVE"[..]] {
        assert!(pcm16(bytes).is_err());
    }
}
