use super::*;

/// (y, x, Java's `Math.atan2(y, x)`) as bits, from Java 24 on a Mac, where macOS's own `atan2`
/// gives one unit in the last place more or less for each.
#[cfg(not(target_os = "android"))]
const JAVA_ATAN2: [(u64, u64, u64); 6] = [
    (0xC063552D00000000, 0xC063DE2800000000, 0xC002F5707D2A1536),
    (0xC07215F760000000, 0x4070348E60000000, 0xBFEAE2C71A81EC45),
    (0xBFE297ABCA418EE4, 0x3FED9833603F03B0, 0xBFE1F31A6F709108),
    (0x405486E5C0000000, 0xC0521A3860000000, 0x4002591D5E6843BC),
    (0x404F800000000000, 0x4057000000000000, 0x3FE336CD78A0C096),
    (0x3FDF43925145BAA8, 0x3FE817A2ED36479E, 0x3FE26AF5C628897A),
];

#[cfg(not(target_os = "android"))]
#[test]
fn atan2_is_javas_to_the_bit() {
    for (y, x, java) in JAVA_ATAN2 {
        let (y, x) = (f64::from_bits(y), f64::from_bits(x));
        assert_eq!(atan2(y, x).to_bits(), java, "atan2({y}, {x})");
    }
}

#[test]
fn atan2_keeps_javas_special_cases() {
    let pi = std::f64::consts::PI;
    assert_eq!(atan2(0.0, 1.0).to_bits(), 0.0f64.to_bits());
    assert_eq!(atan2(-0.0, 1.0).to_bits(), (-0.0f64).to_bits());
    assert_eq!(atan2(0.0, -1.0), pi);
    assert_eq!(atan2(-0.0, -1.0), -pi);
    assert_eq!(atan2(0.0, -0.0), pi);
    assert_eq!(atan2(1.0, 0.0), pi / 2.0);
    assert_eq!(atan2(-1.0, 0.0), -pi / 2.0);
    assert_eq!(atan2(f64::INFINITY, f64::INFINITY), pi / 4.0);
    assert_eq!(atan2(f64::NEG_INFINITY, f64::NEG_INFINITY), -3.0 * pi / 4.0);
    assert_eq!(atan2(1.0, f64::NEG_INFINITY), pi);
    assert_eq!(atan2(-1.0, f64::INFINITY).to_bits(), (-0.0f64).to_bits());
    assert_eq!(atan2(f64::INFINITY, 5.0), pi / 2.0);
    assert!(atan2(f64::NAN, 1.0).is_nan());
    assert!(atan2(1.0, f64::NAN).is_nan());
    assert_eq!(atan2(1e300, 1e-300), pi / 2.0);
    assert_eq!(atan2(1e-300, -1e300), pi);
}

#[test]
fn degrees_round_once() {
    // 180 / π rounded once: Java 8's `angrad * 180.0 / PI` gives 78.7816968304882 here.
    assert_eq!(to_degrees(1.375), 78.781_696_830_488_19);
    assert_eq!(to_degrees(std::f64::consts::PI), 180.0);
}
