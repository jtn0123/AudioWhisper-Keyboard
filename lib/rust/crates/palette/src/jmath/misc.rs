// `Math.hypot`, `Math.cbrt` and `Math.exp` as the JVM computes them: ports of
// fdlibm's e_hypot.c, s_cbrt.c and e_exp.c (fdlibm 5.3). Copyright (C) 1993
// by Sun Microsystems, Inc. All rights reserved. Permission to use, copy,
// modify, and distribute this software is freely granted, provided that this
// notice is preserved.

use super::pow::{P1, P2, P3, P4, P5};
use super::{from_words, hi, lo, with_hi};

/// `sqrt(x*x + y*y)` without undue overflow, bit-identical to `Math.hypot`.
pub(crate) fn hypot(x: f64, y: f64) -> f64 {
    let mut ha = hi(x) & 0x7fff_ffff;
    let mut hb = hi(y) & 0x7fff_ffff;
    let (mut a, mut b) = if hb > ha {
        std::mem::swap(&mut ha, &mut hb);
        (y, x)
    } else {
        (x, y)
    };
    a = with_hi(a, ha);
    b = with_hi(b, hb);
    if (ha - hb) > 0x3c0_0000 {
        return a + b;
    }
    let mut k = 0;
    if ha > 0x5f30_0000 {
        if ha >= 0x7ff0_0000 {
            let mut w = a + b;
            if ((ha & 0xfffff) as u32 | lo(a)) == 0 {
                w = a;
            }
            if ((hb ^ 0x7ff0_0000) as u32 | lo(b)) == 0 {
                w = b;
            }
            return w;
        }
        ha -= 0x2580_0000;
        hb -= 0x2580_0000;
        k += 600;
        a = with_hi(a, ha);
        b = with_hi(b, hb);
    }
    if hb < 0x20b0_0000 {
        if hb <= 0x000f_ffff {
            if (hb as u32 | lo(b)) == 0 {
                return a;
            }
            let t1 = from_words(0x7fd0_0000, 0);
            b *= t1;
            a *= t1;
            k -= 1022;
        } else {
            ha += 0x2580_0000;
            hb += 0x2580_0000;
            k -= 600;
            a = with_hi(a, ha);
            b = with_hi(b, hb);
        }
    }
    let mut w = a - b;
    if w > b {
        let t1 = from_words(ha, 0);
        let t2 = a - t1;
        w = (t1 * t1 - (b * (-b) - t2 * (a + t1))).sqrt();
    } else {
        a = a + a;
        let y1 = from_words(hb, 0);
        let y2 = b - y1;
        let t1 = from_words(ha + 0x0010_0000, 0);
        let t2 = a - t1;
        w = (t1 * y1 - (w * (-w) - (t1 * y2 + t2 * b))).sqrt();
    }
    if k != 0 {
        let t1 = from_words(0x3ff0_0000 + (k << 20), 0);
        t1 * w
    } else {
        w
    }
}

/// Cube root, bit-identical to `Math.cbrt`.
pub(crate) fn cbrt(x: f64) -> f64 {
    const B1: u32 = 715_094_163;
    const B2: u32 = 696_219_795;
    const C: f64 = f64::from_bits(0x3FE1_5F15_F15F_15F1);
    const D: f64 = f64::from_bits(0xBFE6_91DE_2532_C834);
    const E: f64 = f64::from_bits(0x3FF6_A0EA_0EA0_EA0F);
    const F: f64 = f64::from_bits(0x3FF9_B6DB_6DB6_DB6E);
    const G: f64 = f64::from_bits(0x3FD6_DB6D_B6DB_6DB7);

    let mut hx = hi(x) as u32;
    let sign = hx & 0x8000_0000;
    hx ^= sign;
    if hx >= 0x7ff0_0000 {
        return x + x;
    }
    if (hx | lo(x)) == 0 {
        return x;
    }
    let x = with_hi(x, hx as i32);
    let mut t;
    if hx < 0x0010_0000 {
        t = from_words(0x4350_0000, 0);
        t *= x;
        t = with_hi(t, ((hi(t) as u32) / 3 + B2) as i32);
    } else {
        t = from_words((hx / 3 + B1) as i32, 0);
    }
    let mut r = t * t / x;
    let mut s = C + r * t;
    t *= G + F / (s + E + D / s);
    t = from_words(hi(t) + 1, 0);
    s = t * t;
    r = x / s;
    let w = t + t;
    r = (r - t) / (w + r);
    t = t + t * r;
    with_hi(t, (hi(t) as u32 | sign) as i32)
}

/// `e**x`, bit-identical to `Math.exp`.
pub(crate) fn exp(x: f64) -> f64 {
    const HALF: [f64; 2] = [0.5, -0.5];
    const HUGE: f64 = 1.0e300;
    const TWOM1000: f64 = f64::from_bits(0x0170_0000_0000_0000);
    const O_THRESHOLD: f64 = f64::from_bits(0x4086_2E42_FEFA_39EF);
    const U_THRESHOLD: f64 = f64::from_bits(0xC087_4910_D52D_3051);
    const LN2_HI: [f64; 2] = [
        f64::from_bits(0x3FE6_2E42_FEE0_0000),
        f64::from_bits(0xBFE6_2E42_FEE0_0000),
    ];
    const LN2_LO: [f64; 2] = [
        f64::from_bits(0x3DEA_39EF_3579_3C76),
        f64::from_bits(0xBDEA_39EF_3579_3C76),
    ];
    const INVLN2: f64 = f64::from_bits(0x3FF7_1547_652B_82FE);

    let mut x = x;
    let mut hx = hi(x) as u32;
    let xsb = ((hx >> 31) & 1) as usize;
    hx &= 0x7fff_ffff;
    if hx >= 0x4086_2E42 {
        if hx >= 0x7ff0_0000 {
            if ((hx & 0xfffff) | lo(x)) != 0 {
                return x + x;
            }
            return if xsb == 0 { x } else { 0.0 };
        }
        if x > O_THRESHOLD {
            return HUGE * HUGE;
        }
        if x < U_THRESHOLD {
            return TWOM1000 * TWOM1000;
        }
    }
    let (mut hi_part, mut lo_part) = (0.0, 0.0);
    let k: i32;
    if hx > 0x3fd6_2e42 {
        if hx < 0x3FF0_A2B2 {
            hi_part = x - LN2_HI[xsb];
            lo_part = LN2_LO[xsb];
            k = 1 - xsb as i32 - xsb as i32;
        } else {
            k = (INVLN2 * x + HALF[xsb]) as i32;
            let t = f64::from(k);
            hi_part = x - t * LN2_HI[0];
            lo_part = t * LN2_LO[0];
        }
        x = hi_part - lo_part;
    } else if hx < 0x3e30_0000 {
        if HUGE + x > 1.0 {
            return 1.0 + x;
        }
        k = 0;
    } else {
        k = 0;
    }
    let t = x * x;
    let c = x - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));
    if k == 0 {
        return 1.0 - ((x * c) / (c - 2.0) - x);
    }
    let y = 1.0 - ((lo_part - (x * c) / (2.0 - c)) - hi_part);
    if k >= -1021 {
        with_hi(y, hi(y) + (k << 20))
    } else {
        with_hi(y, hi(y) + ((k + 1000) << 20)) * TWOM1000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_values() {
        assert_eq!(hypot(3.0, 4.0), 5.0);
        assert_eq!(cbrt(27.0), 3.0);
        assert_eq!(exp(0.0), 1.0);
    }
}
