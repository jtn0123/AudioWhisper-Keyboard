// `sin` and `cos` as Android's libm (bionic) computes them. On ART,
// `Math.sin` and `Math.cos` call bionic, so these are the answers
// MaterialKolor got on main's keyboard.
//
// A port of fdlibm 5.3's s_sin.c, s_cos.c, k_sin.c, k_cos.c and e_rem_pio2.c
// as FreeBSD's msun revised them and bionic carries them
// (libm/upstream-freebsd/lib/msun/src). FreeBSD evaluates the kernels'
// polynomials in a split order, and its reduction (optimized by Bruce D.
// Evans) takes a fixed multiple of pi/2 off |x| <= 9pi/4 and rounds the
// quotient with `rnint`, the 0x1.8p52 trick. The constants are fdlibm's.
//
// fdlibm: Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
// Developed at SunSoft, a Sun Microsystems, Inc. business.
// Permission to use, copy, modify, and distribute this software is freely
// granted, provided that this notice is preserved.
//
// Rounding. Android builds every platform module, bionic's libm among them,
// with `-ffp-contract=off` (build/soong, cc/config/global.go: "Turn off FMA
// which got enabled by default in clang-r445002"), so each `*`, `+` and `-`
// in the C source rounds on its own. Rust never fuses `a * b + c` either, so
// this file writes the C expressions out unchanged, with no `mul_add`. The
// tests compare it with those C files compiled under that flag.
//
// One departure: arguments of 2^20 * pi/2 and beyond, which bionic hands to
// `__kernel_rem_pio2`, take the medium-size reduction here. No palette input
// comes near them: every angle is a hue in radians, below 8.7.

use super::hi;

const S1: f64 = f64::from_bits(0xBFC5_5555_5555_5549);
const S2: f64 = f64::from_bits(0x3F81_1111_1110_F8A6);
const S3: f64 = f64::from_bits(0xBF2A_01A0_19C1_61D5);
const S4: f64 = f64::from_bits(0x3EC7_1DE3_57B1_FE7D);
const S5: f64 = f64::from_bits(0xBE5A_E5E6_8A2B_9CEB);
const S6: f64 = f64::from_bits(0x3DE5_D93A_5ACF_D57C);
const C1: f64 = f64::from_bits(0x3FA5_5555_5555_554C);
const C2: f64 = f64::from_bits(0xBF56_C16C_16C1_5177);
const C3: f64 = f64::from_bits(0x3EFA_01A0_19CB_1590);
const C4: f64 = f64::from_bits(0xBE92_7E4F_809C_52AD);
const C5: f64 = f64::from_bits(0x3E21_EE9E_BDB4_B1C4);
const C6: f64 = f64::from_bits(0xBDA8_FAE9_BE88_38D4);
/// 53 bits of 2/pi.
const INVPIO2: f64 = f64::from_bits(0x3FE4_5F30_6DC9_C883);
/// The first 33 bits of pi/2, and pi/2 - PIO2_1.
const PIO2_1: f64 = f64::from_bits(0x3FF9_21FB_5440_0000);
const PIO2_1T: f64 = f64::from_bits(0x3DD0_B461_1A62_6331);
/// The second 33 bits of pi/2, and pi/2 - (PIO2_1 + PIO2_2).
const PIO2_2: f64 = f64::from_bits(0x3DD0_B461_1A60_0000);
const PIO2_2T: f64 = f64::from_bits(0x3BA3_198A_2E03_7073);
/// The third 33 bits of pi/2, and pi/2 - (PIO2_1 + PIO2_2 + PIO2_3).
const PIO2_3: f64 = f64::from_bits(0x3BA3_198A_2E00_0000);
const PIO2_3T: f64 = f64::from_bits(0x397B_839A_2520_49C1);
/// 0x1.8p52: `rnint` adds it and takes it away to round to an integer.
const TO_INT: f64 = 6_755_399_441_055_744.0;

/// Sine, as bionic's `sin` (s_sin.c).
pub(crate) fn sin(x: f64) -> f64 {
    let ix = hi(x) & 0x7fff_ffff;
    if ix <= 0x3fe9_21fb {
        // |x| < 2^-26: x itself (C returns it once `(int)x == 0`, which holds).
        if ix < 0x3e50_0000 {
            return x;
        }
        return kernel_sin(x, 0.0, false);
    }
    if ix >= 0x7ff0_0000 {
        return not_a_number(x);
    }
    let (n, y0, y1) = rem_pio2(x);
    match n & 3 {
        0 => kernel_sin(y0, y1, true),
        1 => kernel_cos(y0, y1),
        2 => -kernel_sin(y0, y1, true),
        _ => -kernel_cos(y0, y1),
    }
}

/// Cosine, as bionic's `cos` (s_cos.c).
pub(crate) fn cos(x: f64) -> f64 {
    let ix = hi(x) & 0x7fff_ffff;
    if ix <= 0x3fe9_21fb {
        // |x| < 2^-27 * sqrt(2): one (C returns it once `(int)x == 0`, which holds).
        if ix < 0x3e46_a09e {
            return 1.0;
        }
        return kernel_cos(x, 0.0);
    }
    if ix >= 0x7ff0_0000 {
        return not_a_number(x);
    }
    let (n, y0, y1) = rem_pio2(x);
    match n & 3 {
        0 => kernel_cos(y0, y1),
        1 => -kernel_sin(y0, y1, true),
        2 => -kernel_cos(y0, y1),
        _ => kernel_sin(y0, y1, true),
    }
}

/// What C's `x - x` gives for an infinity or a NaN: the NaN itself, or the
/// default NaN for an infinity.
fn not_a_number(x: f64) -> f64 {
    if x.is_nan() { x } else { f64::NAN }
}

/// `__ieee754_rem_pio2` (e_rem_pio2.c): `x - n * pi/2` as the pair `y0 + y1`,
/// and `n`. Near pi/2, pi, 3pi/2 and 2pi, where taking a fixed multiple off
/// would cancel, it uses the medium-size reduction, as the C does.
fn rem_pio2(x: f64) -> (i32, f64, f64) {
    let hx = hi(x);
    let ix = hx & 0x7fff_ffff;
    if ix <= 0x400f_6a7a {
        // |x| ~<= 5pi/4
        if ix & 0xf_ffff != 0x9_21fb {
            return near(x, hx > 0, if ix <= 0x4002_d97c { 1 } else { 2 });
        }
    } else if ix <= 0x401c_463b {
        // |x| ~<= 9pi/4: 3 up to ~7pi/4, else 4, each but at 3pi/2 and 2pi.
        let (k, cancels) = if ix <= 0x4015_fdbc {
            (3, 0x4012_d97c)
        } else {
            (4, 0x4019_21fb)
        };
        if ix != cancels {
            return near(x, hx > 0, k);
        }
    }
    medium(x, ix)
}

/// The small cases: `x - k * pi/2` in one round, good to 85 bits.
fn near(x: f64, positive: bool, k: i32) -> (i32, f64, f64) {
    let m = f64::from(k);
    let (p, pt) = (m * PIO2_1, m * PIO2_1T);
    if positive {
        let z = x - p;
        let y0 = z - pt;
        (k, y0, (z - y0) - pt)
    } else {
        let z = x + p;
        let y0 = z + pt;
        (-k, y0, (z - y0) + pt)
    }
}

/// The medium case: n = rnint(x * 2/pi), then up to three rounds of
/// `x - n * pi/2`, each taking 33 more bits of pi/2, as cancellation needs.
fn medium(x: f64, ix: i32) -> (i32, f64, f64) {
    let fn_ = (x * INVPIO2 + TO_INT) - TO_INT;
    let n = fn_ as i32;
    let mut r = x - fn_ * PIO2_1;
    let mut w = fn_ * PIO2_1T;
    let j = ix >> 20;
    let mut y0 = r - w;
    if j - ((hi(y0) >> 20) & 0x7ff) > 16 {
        let t = r;
        w = fn_ * PIO2_2;
        r = t - w;
        w = fn_ * PIO2_2T - ((t - r) - w);
        y0 = r - w;
        if j - ((hi(y0) >> 20) & 0x7ff) > 49 {
            let t = r;
            w = fn_ * PIO2_3;
            r = t - w;
            w = fn_ * PIO2_3T - ((t - r) - w);
            y0 = r - w;
        }
    }
    (n, y0, (r - y0) - w)
}

/// `__kernel_sin` (k_sin.c) on ~[-pi/4, pi/4]; `y` is the tail of `x`, and
/// zero unless `tail`.
fn kernel_sin(x: f64, y: f64, tail: bool) -> f64 {
    let z = x * x;
    let w = z * z;
    let r = S2 + z * (S3 + z * S4) + z * w * (S5 + z * S6);
    let v = z * x;
    if tail {
        x - ((z * (0.5 * y - v * r) - y) - v * S1)
    } else {
        x + v * (S1 + z * r)
    }
}

/// `__kernel_cos` (k_cos.c) on ~[-pi/4, pi/4]; `y` is the tail of `x`.
fn kernel_cos(x: f64, y: f64) -> f64 {
    let z = x * x;
    let w = z * z;
    let r = z * (C1 + z * (C2 + z * C3)) + w * w * (C4 + z * (C5 + z * C6));
    let hz = 0.5 * z;
    let w = 1.0 - hz;
    w + (((1.0 - w) - hz) + (z * r - x * y))
}

#[cfg(test)]
mod tests {
    use super::{cos, sin};

    /// (x, sin x, cos x) as bits, from bionic's s_sin.c, s_cos.c, k_sin.c, k_cos.c and
    /// e_rem_pio2.c compiled with `-ffp-contract=off`: one argument or more per branch, and
    /// three the sweep below misses, which a moved cut-off for sin, a last bit off pio2_1t or
    /// a skipped third reduction round would change.
    const BIONIC: [(u64, u64, u64); 30] = [
        (0x3E112E0BE826D695, 0x3E112E0BE826D695, 0x3FF0000000000000), // 1e-9: below both tiny cut-offs
        (0x3E50000000000000, 0x3E50000000000000, 0x3FEFFFFFFFFFFFFF), // 2^-26: sin's cut-off, the kernel
        (0x3E57137450D8F7C0, 0x3E57137450D8F7BF, 0x3FEFFFFFFFFFFFFE), // 2.1e-8: sin's kernel, not x
        (0x3E46A09E00000000, 0x3E46A09E00000000, 0x3FF0000000000000), // cos's cut-off, the kernel
        (0x3FE0000000000000, 0x3FDEAEE8744B05F0, 0x3FEC1528065B7D50), // 0.5: kernels only
        (0xBFE6666666666666, 0xBFE49D6E694619B8, 0x3FE87996529F9D93), // -0.7: kernels only
        (0x3FF0000000000000, 0x3FEAED548F090CEE, 0x3FE14A280FB5068C), // 1.0: x - pi/2
        (0xBFF3333333333333, 0xBFEDD343A21A55C4, 0x3FD730DE943B79D4), // -1.2: x + pi/2
        (0x3FF921FAB0047BDD, 0x3FEFFFFFFFFFF96A, 0x3EA487F62768D1AB), // pi/2 - 6e-7: pio2_1t's last bit
        (0x4004000000000000, 0x3FE326AF0DCFCAB0, 0xBFE9A2F7EF858B7D), // 2.5: x - 2(pi/2)
        (0xC008000000000000, 0xBFC210386DB6D55B, 0xBFEFAE04BE85E5D2), // -3.0: x + 2(pi/2)
        (0x4012000000000000, 0xBFEF47ED3DC74080, 0xBFCAFB5B54583D6A), // 4.5: x - 3(pi/2)
        (0xC014000000000000, 0x3FEEAF81F5E09933, 0x3FD22785706B4AD9), // -5.0: x + 3(pi/2)
        (0x4018000000000000, 0xBFD1E1F18AB0A2C0, 0x3FEEB9B7097822F5), // 6.0: x - 4(pi/2)
        (0xC01A000000000000, 0xBFCB890D40C6CB3C, 0x3FEF4034B145C3C8), // -6.5: x + 4(pi/2)
        (0x3FF921FB54442D18, 0x3FF0000000000000, 0x3C91A62633145C07), // pi/2: cancels, medium
        (0xC00921FB54442D18, 0xBCA1A62633145C07, 0xBFF0000000000000), // -pi: cancels, medium
        (0x4012D97C7F3321D2, 0xBFF0000000000000, 0xBCAA79394C9E8A0A), // 3pi/2: cancels, medium
        (0xC01921FB54442D18, 0x3CB1A62633145C07, 0x3FF0000000000000), // -2pi: cancels, medium
        (0x401E000000000000, 0x3FEE041886FCAE30, 0x3FD62F45E66F5C2F), // 7.5: medium, n = 5
        (0xC020000000000000, 0xBFEFA8D2A028CF7B, 0xBFC29FBEBF632F94), // -8.0: medium, n = -5
        (0x4021333333333333, 0x3FE7802E57B4B12D, 0xBFE5B8131ACC20FB), // 8.6: medium, n = 5
        (0x401F6A7A2955385E, 0x3FF0000000000000, 0x3CB60FAFBFD97309), // 5pi/2: medium, second round
        (0x4059000000000000, 0xBFE03425B78C4DB8, 0x3FEB981DBF665FDF), // 100: medium
        (0x40AB9328A0144CFA, 0xBFF0000000000000, 0xBCF3A565295ECF02), // 2247(pi/2): the third round
        (0x40F86A0000000000, 0x3FA24DAA9C527E96, 0xBFEFFAC3841B3DA7), // 1e5: medium
        (0x4136E36000000000, 0x3FE05A19F9BF8B6C, 0xBFEB81B10301DAB6), // 1.5e6: medium, near its end
        (0x0000000000000000, 0x0000000000000000, 0x3FF0000000000000), // +0
        (0x8000000000000000, 0x8000000000000000, 0x3FF0000000000000), // -0
        (0x7FF0000000000000, 0x7FF8000000000000, 0x7FF8000000000000), // +inf: NaN
    ];

    /// FNV-1a over the bits of `sin x` then `cos x` for 200,001 evenly spaced `x` in [-9, 9],
    /// then for the 129 `x` within 64 ulps of each nonzero `k * pi/4`, `k` from -12 to 12,
    /// from the same C build. The hue angles live in that range.
    const BIONIC_SWEEP: u64 = 0x69ED_E9D1_16C9_F5C5;

    #[test]
    fn a_sweep_hashes_as_bionic_does() {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        let mut both = |x: f64| {
            for v in [sin(x), cos(x)] {
                for byte in v.to_bits().to_le_bytes() {
                    hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
                }
            }
        };
        for i in 0..=200_000_u32 {
            both(-9.0 + 18.0 * f64::from(i) / 200_000.0);
        }
        let quarter = std::f64::consts::FRAC_PI_4;
        for k in (-12..=12_i32).filter(|&k| k != 0) {
            let m = (f64::from(k) * quarter).to_bits();
            for d in -64..=64_i64 {
                both(f64::from_bits(m.wrapping_add_signed(d)));
            }
        }
        assert_eq!(hash, BIONIC_SWEEP, "the sweep's hash, against bionic's");
    }

    #[test]
    fn every_branch_answers_as_bionic_does() {
        let same = |got: f64, want: u64| {
            let want = f64::from_bits(want);
            got.to_bits() == want.to_bits() || (got.is_nan() && want.is_nan())
        };
        for (x, want_sin, want_cos) in BIONIC {
            let x = f64::from_bits(x);
            assert!(
                same(sin(x), want_sin),
                "sin({x:e}) = {:016x}, bionic {want_sin:016x}",
                sin(x).to_bits()
            );
            assert!(
                same(cos(x), want_cos),
                "cos({x:e}) = {:016x}, bionic {want_cos:016x}",
                cos(x).to_bits()
            );
        }
    }
}
