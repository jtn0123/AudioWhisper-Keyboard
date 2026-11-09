use super::*;
use crate::utf16;

fn float(text: &str) -> Option<f32> {
    to_float_or_null(&utf16(text))
}

fn bits(text: &str) -> Option<u32> {
    float(text).map(f32::to_bits)
}

#[test]
fn decimals_read_as_kotlin_reads_them() {
    assert_eq!(float("200"), Some(200.0));
    assert_eq!(float("-1.5"), Some(-1.5));
    assert_eq!(float("+.5"), Some(0.5));
    assert_eq!(float("5."), Some(5.0));
    assert_eq!(float("1e3"), Some(1000.0));
    assert_eq!(float("1E-3"), Some(0.001));
    assert_eq!(float("1e+3f"), Some(1000.0));
    assert_eq!(float("2D"), Some(2.0));
    assert_eq!(float("0007.25"), Some(7.25));
    assert_eq!(float("0.1"), Some(0.1));
    assert_eq!(bits("-0"), Some((-0.0f32).to_bits()));
    assert_eq!(float("."), None);
    assert_eq!(float(""), None);
    assert_eq!(float("e3"), None);
    assert_eq!(float("1e"), None);
    assert_eq!(float("1e+"), None);
    assert_eq!(float("1.2.3"), None);
    assert_eq!(float("1ff"), None);
    assert_eq!(float("1 2"), None);
    assert_eq!(float("--1"), None);
    assert_eq!(float("1_000"), None);
    assert_eq!(float("١"), None);
    assert_eq!(float("２"), None);
}

#[test]
fn only_units_up_to_space_are_trimmed() {
    assert_eq!(float(" \t\n\u{0}\u{1F}42 \r"), Some(42.0));
    assert_eq!(float("\u{A0}42"), None);
    assert_eq!(float("42\u{2007}"), None);
    assert_eq!(float("- 1"), None);
}

#[test]
fn names_need_their_exact_case() {
    assert!(float("NaN").unwrap().is_nan());
    assert!(float("-NaN").unwrap().is_nan());
    assert_eq!(float("Infinity"), Some(f32::INFINITY));
    assert_eq!(float("-Infinity"), Some(f32::NEG_INFINITY));
    assert_eq!(float("nan"), None);
    assert_eq!(float("inf"), None);
    assert_eq!(float("InfinityF"), None);
    assert_eq!(float("NaNd"), None);
}

#[test]
fn far_exponents_saturate() {
    assert_eq!(float("1e39"), Some(f32::INFINITY));
    assert_eq!(float("1e-46"), Some(0.0));
    assert_eq!(float("1e99999999999999999999"), Some(f32::INFINITY));
    assert_eq!(float("1e-99999999999999999999"), Some(0.0));
    assert_eq!(float("0e99999999999999999999"), Some(0.0));
    let tiny = format!("0.{}1e999", "0".repeat(1000));
    assert_eq!(float(&tiny), Some(0.01));
    let long = format!("1{}e-1000", "0".repeat(1000));
    assert_eq!(float(&long), Some(1.0));
}

#[test]
fn decimals_round_to_nearest_even() {
    // 2^24 + 1 is halfway between two floats; even wins, and anything after tips it.
    assert_eq!(float("16777217"), Some(16_777_216.0));
    assert_eq!(float("16777217.000000000000000000001"), Some(16_777_218.0));
    assert_eq!(float("16777219"), Some(16_777_220.0));
    assert_eq!(bits("1.4e-45"), Some(1));
    assert_eq!(bits("7e-46"), Some(0));
    assert_eq!(bits("7.1e-46"), Some(1));
    assert_eq!(float("3.4028235e38"), Some(f32::MAX));
    assert_eq!(float("3.40282356e38"), Some(f32::MAX));
    assert_eq!(float("3.40282357e38"), Some(f32::INFINITY));
    assert_eq!(float("3.4028236e38"), Some(f32::INFINITY));
}

#[test]
fn hexadecimals_need_an_exponent() {
    assert_eq!(float("0x1p0"), Some(1.0));
    assert_eq!(float("0X1.8P1"), Some(3.0));
    assert_eq!(float("-0x.8p0"), Some(-0.5));
    assert_eq!(float("0xAp-1f"), Some(5.0));
    assert_eq!(float("0x0p99999999999"), Some(0.0));
    assert_eq!(float("0x1"), None);
    assert_eq!(float("0x.p1"), None);
    assert_eq!(float("0xp1"), None);
    assert_eq!(float("0x1p"), None);
    assert_eq!(float("0x1pA"), None);
    assert_eq!(float("0x1g0p0"), None);
    assert_eq!(float("x1p0"), None);
    assert_eq!(float("00x1p0"), None);
}

#[test]
fn hexadecimals_round_to_nearest_even() {
    assert_eq!(bits("0x1.000001p0"), Some(1.0f32.to_bits()));
    assert_eq!(bits("0x1.000003p0"), Some(1.0f32.to_bits() + 2));
    assert_eq!(
        bits("0x1.0000010000000000000001p0"),
        Some(1.0f32.to_bits() + 1)
    );
    assert_eq!(bits("0x1.fffffffp0"), Some(2.0f32.to_bits()));
    assert_eq!(bits("0x1p-149"), Some(1));
    assert_eq!(bits("0x1p-150"), Some(0));
    assert_eq!(bits("0x1.1p-150"), Some(1));
    assert_eq!(bits("0x3p-151"), Some(1));
    assert_eq!(bits("0x1p-126"), Some(0x80_0000));
    assert_eq!(bits("0x0.fffffffp-126"), Some(0x80_0000));
    assert_eq!(float("0x1.fffffep127"), Some(f32::MAX));
    assert_eq!(float("0x1.ffffffp127"), Some(f32::INFINITY));
    assert_eq!(float("0x1p128"), Some(f32::INFINITY));
    assert_eq!(float("0x1p-99999999999"), Some(0.0));
    assert_eq!(float("0x00000000000000000001p0"), Some(1.0));
    assert_eq!(float("0x10000000000000000000p-76"), Some(1.0));
}
