use keys::codes::*;
/// Complete audio/haptic decision. Returned values are effect, volume bits, haptic constant,
/// vibration factor bits, and respect-system flag. Disabled effects use -1.
pub fn plan(v: &[i64]) -> Result<Vec<i64>, &'static str> {
    if v.len() != 12 {
        return Err("wrong feedback field count");
    }
    // event, code, audio feature, haptic feature, audio enabled, haptic enabled,
    // audio respects system, system audio, volume, haptic interface, haptic respects system, vibrator present.
    let event = v[0];
    if !(0..=4).contains(&event) {
        return Err("unknown feedback event");
    }
    let audio: f64 = match event {
        0 => 1.0,
        1 | 3 => 0.7,
        _ => 0.4,
    };
    let haptic: f64 = match event {
        0 => 1.0,
        1 | 3 => 0.4,
        _ => 0.05,
    };
    let volume = (v[8] as f64 * audio) / 100.0;
    let sound =
        if v[2] != 0 && v[4] != 0 && (v[6] == 0 || v[7] != 0) && (0.01..=1.0).contains(&volume) {
            match v[1] as i32 {
                DELETE => 7,
                ENTER => 8,
                SPACE => 6,
                _ => 5,
            }
        } else {
            -1
        };
    let vibration = if v[3] != 0 && v[5] != 0 && v[11] != 0 {
        if v[9] != 0 {
            if haptic < 1.0 { 9 } else { 3 }
        } else {
            0
        }
    } else {
        -1
    };
    Ok(vec![
        sound,
        i64::from((volume as f32).to_bits() as i32),
        vibration,
        haptic.to_bits() as i64,
        v[10],
    ])
}
