// `Math.atan2` as the JVM computes it: a port of fdlibm's e_atan2.c and
// s_atan.c (fdlibm 5.3). Copyright (C) 1993 by Sun Microsystems, Inc. All
// rights reserved. Permission to use, copy, modify, and distribute this
// software is freely granted, provided that this notice is preserved.

use super::{hi, lo, with_hi};

const ATAN_HI: [f64; 4] = [
    f64::from_bits(0x3FDD_AC67_0561_BB4F),
    f64::from_bits(0x3FE9_21FB_5444_2D18),
    f64::from_bits(0x3FEF_730B_D281_F69B),
    f64::from_bits(0x3FF9_21FB_5444_2D18),
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
const TINY: f64 = 1.0e-300;
const PI_O_4: f64 = f64::from_bits(0x3FE9_21FB_5444_2D18);
const PI_O_2: f64 = f64::from_bits(0x3FF9_21FB_5444_2D18);
const PI: f64 = f64::from_bits(0x4009_21FB_5444_2D18);
const PI_LO: f64 = f64::from_bits(0x3CA1_A626_3314_5C07);

/// Arc tangent, bit-identical to fdlibm's `atan`.
fn atan(x: f64) -> f64 {
    let hx = hi(x);
    let ix = hx & 0x7fff_ffff;
    let mut x = x;
    let id: i32;
    if ix >= 0x4410_0000 {
        if ix > 0x7ff0_0000 || (ix == 0x7ff0_0000 && lo(x) != 0) {
            return x + x;
        }
        return if hx > 0 {
            ATAN_HI[3] + ATAN_LO[3]
        } else {
            -ATAN_HI[3] - ATAN_LO[3]
        };
    }
    if ix < 0x3fdc_0000 {
        if ix < 0x3e20_0000 && 1.0e300 + x > 1.0 {
            return x;
        }
        id = -1;
    } else {
        x = x.abs();
        if ix < 0x3ff3_0000 {
            if ix < 0x3fe6_0000 {
                id = 0;
                x = (2.0 * x - 1.0) / (2.0 + x);
            } else {
                id = 1;
                x = (x - 1.0) / (x + 1.0);
            }
        } else if ix < 0x4003_8000 {
            id = 2;
            x = (x - 1.5) / (1.0 + 1.5 * x);
        } else {
            id = 3;
            x = -1.0 / x;
        }
    }
    let z = x * x;
    let w = z * z;
    let s1 = z * (AT[0] + w * (AT[2] + w * (AT[4] + w * (AT[6] + w * (AT[8] + w * AT[10])))));
    let s2 = w * (AT[1] + w * (AT[3] + w * (AT[5] + w * (AT[7] + w * AT[9]))));
    if id < 0 {
        return x - x * (s1 + s2);
    }
    let id = id as usize;
    let z = ATAN_HI[id] - ((x * (s1 + s2) - ATAN_LO[id]) - x);
    if hx < 0 { -z } else { z }
}

/// `atan2(y, x)`, bit-identical to `Math.atan2`.
pub(crate) fn atan2(y: f64, x: f64) -> f64 {
    let hx = hi(x);
    let ix = hx & 0x7fff_ffff;
    let lx = lo(x);
    let hy = hi(y);
    let iy = hy & 0x7fff_ffff;
    let ly = lo(y);
    if (ix as u32 | ((lx | lx.wrapping_neg()) >> 31)) > 0x7ff0_0000
        || (iy as u32 | ((ly | ly.wrapping_neg()) >> 31)) > 0x7ff0_0000
    {
        return x + y;
    }
    if ((hx.wrapping_sub(0x3ff0_0000)) as u32 | lx) == 0 {
        return atan(y);
    }
    let m = ((hy >> 31) & 1) | ((hx >> 30) & 2);
    if (iy as u32 | ly) == 0 {
        return match m {
            0 | 1 => y,
            2 => PI + TINY,
            _ => -PI - TINY,
        };
    }
    if (ix as u32 | lx) == 0 {
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
        1 => with_hi(z, hi(z) ^ (0x8000_0000u32 as i32)),
        2 => PI - (z - PI_LO),
        _ => (z - PI_LO) - PI,
    }
}
