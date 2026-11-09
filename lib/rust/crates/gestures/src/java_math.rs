//! `java.lang.Math`'s `atan2` and `toDegrees`, to the bit, as Kotlin gets them where this runs.
//!
//! A swipe's direction is a sector of the angle `atan2` gives, so one unit in the last place
//! decides a swipe that lands on a sector's edge. The two platforms differ:
//!
//! - On Android, `Math.atan2` is bionic's `atan2`, which is what Rust's [`f64::atan2`] calls there.
//! - On a computer's JVM, `Math.atan2` is `StrictMath.atan2`, fdlibm's, which a computer's own
//!   `atan2` doesn't match: macOS's differs from it in the last bit for about one angle in six. So
//!   the library the JVM tests load carries fdlibm's, ported below from `e_atan2.c` and `s_atan.c`
//!   (as Java's `FdLibm` has them).
//!
//! `Math.toDegrees` is a multiplication by `180 / π` rounded once, on both.

/// `Math.toDegrees`: Java 9 and later multiply by this constant rather than computing
/// `angrad * 180.0 / PI`, which rounds twice.
const RADIANS_TO_DEGREES: f64 = 57.295_779_513_082_32;

/// `Math.toDegrees(radians)`.
pub fn to_degrees(radians: f64) -> f64 {
    radians * RADIANS_TO_DEGREES
}

/// `Math.atan2(y, x)` on Android: bionic's.
#[cfg(target_os = "android")]
pub fn atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

/// `Math.atan2(y, x)` on a computer's JVM: fdlibm's.
#[cfg(not(target_os = "android"))]
pub fn atan2(y: f64, x: f64) -> f64 {
    fdlibm::atan2(y, x)
}

#[cfg(not(target_os = "android"))]
pub(crate) mod fdlibm {
    const ATAN_HI: [f64; 4] = [
        f64::from_bits(0x3FDD_AC67_0561_BB4F), // atan(0.5)
        f64::from_bits(0x3FE9_21FB_5444_2D18), // atan(1.0)
        f64::from_bits(0x3FEF_730B_D281_F69B), // atan(1.5)
        f64::from_bits(0x3FF9_21FB_5444_2D18), // atan(inf)
    ];
    const ATAN_LO: [f64; 4] = [
        f64::from_bits(0x3C7A_2B7F_222F_65E2),
        f64::from_bits(0x3C81_A626_3314_5C07),
        f64::from_bits(0x3C70_0788_7AF0_CBBD),
        f64::from_bits(0x3C91_A626_3314_5C07),
    ];
    const AT: [f64; 11] = [
        f64::from_bits(0x3FD5_5555_5555_550D),
        f64::from_bits(0xBFC9_9999_9998_EBC4),
        f64::from_bits(0x3FC2_4924_9200_83FF),
        f64::from_bits(0xBFBC_71C6_FE23_1671),
        f64::from_bits(0x3FB7_45CD_C54C_206E),
        f64::from_bits(0xBFB3_B0F2_AF74_9A6D),
        f64::from_bits(0x3FB1_0D66_A0D0_3D51),
        f64::from_bits(0xBFAD_DE2D_52DE_FD9A),
        f64::from_bits(0x3FA9_7B4B_2476_0DEB),
        f64::from_bits(0xBFA2_B444_2C6A_6C2F),
        f64::from_bits(0x3F90_AD3A_E322_DA11),
    ];
    const PI_O_4: f64 = f64::from_bits(0x3FE9_21FB_5444_2D18);
    const PI_O_2: f64 = f64::from_bits(0x3FF9_21FB_5444_2D18);
    const PI: f64 = f64::from_bits(0x4009_21FB_5444_2D18);
    const PI_LO: f64 = f64::from_bits(0x3CA1_A626_3314_5C07);
    const TINY: f64 = 1.0e-300;

    /// The high 32 bits, as fdlibm's `__HI`.
    fn hi(x: f64) -> i32 {
        (x.to_bits() >> 32) as i32
    }

    /// The low 32 bits, as fdlibm's `__LO`.
    fn lo(x: f64) -> u32 {
        x.to_bits() as u32
    }

    /// fdlibm's `atan`.
    pub fn atan(x: f64) -> f64 {
        let hx = hi(x);
        let ix = hx & 0x7fff_ffff;
        if ix >= 0x4410_0000 {
            // |x| >= 2^66
            if x.is_nan() {
                return x + x;
            }
            return if hx > 0 {
                ATAN_HI[3] + ATAN_LO[3]
            } else {
                -ATAN_HI[3] - ATAN_LO[3]
            };
        }
        let (id, x) = if ix < 0x3fdc_0000 {
            // |x| < 0.4375
            if ix < 0x3e20_0000 {
                // |x| < 2^-29: atan(x) is x
                return x;
            }
            (None, x)
        } else {
            let x = x.abs();
            if ix < 0x3ff3_0000 {
                if ix < 0x3fe6_0000 {
                    (Some(0), (2.0 * x - 1.0) / (2.0 + x))
                } else {
                    (Some(1), (x - 1.0) / (x + 1.0))
                }
            } else if ix < 0x4003_8000 {
                (Some(2), (x - 1.5) / (1.0 + 1.5 * x))
            } else {
                (Some(3), -1.0 / x)
            }
        };
        let z = x * x;
        let w = z * z;
        let s1 = z * (AT[0] + w * (AT[2] + w * (AT[4] + w * (AT[6] + w * (AT[8] + w * AT[10])))));
        let s2 = w * (AT[1] + w * (AT[3] + w * (AT[5] + w * (AT[7] + w * AT[9]))));
        match id {
            None => x - x * (s1 + s2),
            Some(id) => {
                let z = ATAN_HI[id] - ((x * (s1 + s2) - ATAN_LO[id]) - x);
                if hx < 0 { -z } else { z }
            }
        }
    }

    /// fdlibm's `__ieee754_atan2`.
    pub fn atan2(y: f64, x: f64) -> f64 {
        if x.is_nan() || y.is_nan() {
            return x + y;
        }
        let (hx, lx) = (hi(x), lo(x));
        let (hy, ly) = (hi(y), lo(y));
        let (ix, iy) = (hx & 0x7fff_ffff, hy & 0x7fff_ffff);
        if hx.wrapping_sub(0x3ff0_0000) as u32 | lx == 0 {
            // x is 1.0
            return atan(y);
        }
        // 2 * sign(x) + sign(y)
        let m = ((hy >> 31) & 1) | ((hx >> 30) & 2);
        if iy as u32 | ly == 0 {
            return match m {
                0 | 1 => y,
                2 => PI + TINY,
                _ => -PI - TINY,
            };
        }
        if ix as u32 | lx == 0 {
            return if hy < 0 {
                -PI_O_2 - TINY
            } else {
                PI_O_2 + TINY
            };
        }
        if ix == 0x7ff0_0000 {
            return if iy == 0x7ff0_0000 {
                match m {
                    0 => PI_O_4 + TINY,
                    1 => -PI_O_4 - TINY,
                    2 => 3.0 * PI_O_4 + TINY,
                    _ => -3.0 * PI_O_4 - TINY,
                }
            } else {
                match m {
                    0 => 0.0,
                    1 => -0.0,
                    2 => PI + TINY,
                    _ => -PI - TINY,
                }
            };
        }
        if iy == 0x7ff0_0000 {
            return if hy < 0 {
                -PI_O_2 - TINY
            } else {
                PI_O_2 + TINY
            };
        }
        let k = (iy - ix) >> 20;
        let z = if k > 60 {
            PI_O_2 + 0.5 * PI_LO
        } else if hx < 0 && k < -60 {
            0.0
        } else {
            atan((y / x).abs())
        };
        match m {
            0 => z,
            1 => -z,
            2 => PI - (z - PI_LO),
            _ => (z - PI_LO) - PI,
        }
    }
}

#[cfg(test)]
mod tests;
