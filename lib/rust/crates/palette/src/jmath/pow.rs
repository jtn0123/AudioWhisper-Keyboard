// `Math.pow` / `StrictMath.pow` as the JVM computes it: a port of fdlibm's
// e_pow.c (fdlibm 5.3). Copyright (C) 1993-2004 by Sun Microsystems, Inc.
// All rights reserved. Permission to use, copy, modify, and distribute this
// software is freely granted, provided that this notice is preserved.

use super::{from_words, hi, lo, with_hi, with_lo};

const BP: [f64; 2] = [1.0, 1.5];
const DP_H: [f64; 2] = [0.0, f64::from_bits(0x3FE2_B803_4000_0000)];
const DP_L: [f64; 2] = [0.0, f64::from_bits(0x3E4C_FDEB_43CF_D006)];
const TWO53: f64 = f64::from_bits(0x4340_0000_0000_0000);
const HUGE: f64 = 1.0e300;
const TINY: f64 = 1.0e-300;
const L1: f64 = f64::from_bits(0x3FE3_3333_3333_3303);
const L2: f64 = f64::from_bits(0x3FDB_6DB6_DB6F_ABFF);
const L3: f64 = f64::from_bits(0x3FD5_5555_518F_264D);
const L4: f64 = f64::from_bits(0x3FD1_7460_A91D_4101);
const L5: f64 = f64::from_bits(0x3FCD_864A_93C9_DB65);
const L6: f64 = f64::from_bits(0x3FCA_7E28_4A45_4EEF);
pub(super) const P1: f64 = f64::from_bits(0x3FC5_5555_5555_553E);
pub(super) const P2: f64 = f64::from_bits(0xBF66_C16C_16BE_BD93);
pub(super) const P3: f64 = f64::from_bits(0x3F11_566A_AF25_DE2C);
pub(super) const P4: f64 = f64::from_bits(0xBEBB_BD41_C5D2_6BF1);
pub(super) const P5: f64 = f64::from_bits(0x3E66_3769_72BE_A4D0);
const LG2: f64 = f64::from_bits(0x3FE6_2E42_FEFA_39EF);
const LG2_H: f64 = f64::from_bits(0x3FE6_2E43_0000_0000);
const LG2_L: f64 = f64::from_bits(0xBE20_5C61_0CA8_6C39);
const OVT: f64 = f64::from_bits(0x3C97_1547_652B_82FE);
const CP: f64 = f64::from_bits(0x3FEE_C709_DC3A_03FD);
const CP_H: f64 = f64::from_bits(0x3FEE_C709_E000_0000);
const CP_L: f64 = f64::from_bits(0xBE3E_2FE0_145B_01F5);
const IVLN2: f64 = f64::from_bits(0x3FF7_1547_652B_82FE);
const THIRD: f64 = f64::from_bits(0x3FD5_5555_5555_5555);
const IVLN2_H: f64 = f64::from_bits(0x3FF7_1547_6000_0000);
const IVLN2_L: f64 = f64::from_bits(0x3E54_AE0B_F85D_DF44);

/// `x` raised to `y`, bit-identical to the JVM's `Math.pow`.
pub(crate) fn pow(x: f64, y: f64) -> f64 {
    let hx = hi(x);
    let lx = lo(x);
    let hy = hi(y);
    let ly = lo(y);
    let mut ix = hx & 0x7fff_ffff;
    let iy = hy & 0x7fff_ffff;

    // y == 0: x**0 = 1
    if (iy as u32 | ly) == 0 {
        return 1.0;
    }
    // NaN
    if ix > 0x7ff0_0000
        || (ix == 0x7ff0_0000 && lx != 0)
        || iy > 0x7ff0_0000
        || (iy == 0x7ff0_0000 && ly != 0)
    {
        return x + y;
    }
    let yisint = y_int_kind(hx, iy, ly);
    if ly == 0 {
        if iy == 0x7ff0_0000 {
            // y is +-inf
            return if ((ix - 0x3ff0_0000) as u32 | lx) == 0 {
                f64::NAN
            } else if ix >= 0x3ff0_0000 {
                if hy >= 0 { y } else { 0.0 }
            } else if hy < 0 {
                -y
            } else {
                0.0
            };
        }
        if iy == 0x3ff0_0000 {
            return if hy < 0 { 1.0 / x } else { x };
        }
        if hy == 0x4000_0000 {
            return x * x;
        }
        if hy == 0x3fe0_0000 && hx >= 0 {
            return x.sqrt();
        }
    }
    let mut ax = x.abs();
    if lx == 0 && (ix == 0x7ff0_0000 || ix == 0 || ix == 0x3ff0_0000) {
        let mut z = ax;
        if hy < 0 {
            z = 1.0 / z;
        }
        if hx < 0 {
            if ((ix - 0x3ff0_0000) | yisint) == 0 {
                z = f64::NAN;
            } else if yisint == 1 {
                z = -z;
            }
        }
        return z;
    }
    let mut n = (hx >> 31) + 1;
    if (n | yisint) == 0 {
        return f64::NAN;
    }
    let mut s = 1.0;
    if (n | (yisint - 1)) == 0 {
        s = -1.0;
    }

    let (t1, t2);
    if iy > 0x41e0_0000 {
        // |y| > 2**31
        if iy > 0x43f0_0000 {
            if ix <= 0x3fef_ffff {
                return if hy < 0 { HUGE * HUGE } else { TINY * TINY };
            }
            if ix >= 0x3ff0_0000 {
                return if hy > 0 { HUGE * HUGE } else { TINY * TINY };
            }
        }
        if ix < 0x3fef_ffff {
            return if hy < 0 {
                s * HUGE * HUGE
            } else {
                s * TINY * TINY
            };
        }
        if ix > 0x3ff0_0000 {
            return if hy > 0 {
                s * HUGE * HUGE
            } else {
                s * TINY * TINY
            };
        }
        let t = ax - 1.0;
        let w = (t * t) * (0.5 - t * (THIRD - t * 0.25));
        let u = IVLN2_H * t;
        let v = t * IVLN2_L - w * IVLN2;
        t1 = with_lo(u + v, 0);
        t2 = v - (t1 - u);
    } else {
        n = 0;
        if ix < 0x0010_0000 {
            ax *= TWO53;
            n -= 53;
            ix = hi(ax);
        }
        n += (ix >> 20) - 0x3ff;
        let j = ix & 0x000f_ffff;
        ix = j | 0x3ff0_0000;
        let k: usize;
        if j <= 0x3988E {
            k = 0;
        } else if j < 0xBB67A {
            k = 1;
        } else {
            k = 0;
            n += 1;
            ix -= 0x0010_0000;
        }
        ax = with_hi(ax, ix);
        (t1, t2) = log2_parts(ax, ix, k, n);
    }
    finish(y, t1, t2, s)
}

/// Classifies `y` when `x < 0`: 0 not an integer, 1 odd, 2 even.
fn y_int_kind(hx: i32, iy: i32, ly: u32) -> i32 {
    let mut yisint = 0;
    if hx < 0 {
        if iy >= 0x4340_0000 {
            yisint = 2;
        } else if iy >= 0x3ff0_0000 {
            let k = (iy >> 20) - 0x3ff;
            if k > 20 {
                let j = ly >> (52 - k);
                if (j << (52 - k)) == ly {
                    yisint = 2 - (j & 1) as i32;
                }
            } else if ly == 0 {
                let j = iy >> (20 - k);
                if (j << (20 - k)) == iy {
                    yisint = 2 - (j & 1);
                }
            }
        }
    }
    yisint
}

/// log2(ax) as `t1 + t2`, for the normalised `ax` with high word `ix`.
fn log2_parts(ax: f64, ix: i32, k: usize, n: i32) -> (f64, f64) {
    let u = ax - BP[k];
    let v = 1.0 / (ax + BP[k]);
    let ss = u * v;
    let s_h = with_lo(ss, 0);
    let t_h = from_words(
        ((ix >> 1) | 0x2000_0000) + 0x0008_0000 + ((k as i32) << 18),
        0,
    );
    let t_l = ax - (t_h - BP[k]);
    let s_l = v * ((u - s_h * t_h) - s_h * t_l);
    let mut s2 = ss * ss;
    let mut r = s2 * s2 * (L1 + s2 * (L2 + s2 * (L3 + s2 * (L4 + s2 * (L5 + s2 * L6)))));
    r += s_l * (s_h + ss);
    s2 = s_h * s_h;
    let t_h = with_lo(3.0 + s2 + r, 0);
    let t_l = r - ((t_h - 3.0) - s2);
    let u = s_h * t_h;
    let v = s_l * t_h + t_l * ss;
    let p_h = with_lo(u + v, 0);
    let p_l = v - (p_h - u);
    let z_h = CP_H * p_h;
    let z_l = CP_L * p_h + p_l * CP + DP_L[k];
    let t = f64::from(n);
    let t1 = with_lo(((z_h + z_l) + DP_H[k]) + t, 0);
    let t2 = z_l - (((t1 - t) - DP_H[k]) - z_h);
    (t1, t2)
}

/// Computes `s * 2**(y * (t1 + t2))`.
fn finish(y: f64, t1: f64, t2: f64, s: f64) -> f64 {
    let y1 = with_lo(y, 0);
    let p_l = (y - y1) * t1 + y * t2;
    let mut p_h = y1 * t1;
    let z = p_l + p_h;
    let j = hi(z);
    let i = lo(z) as i32;
    if j >= 0x4090_0000 {
        if ((j - 0x4090_0000) | i) != 0 || p_l + OVT > z - p_h {
            return s * HUGE * HUGE;
        }
    } else if (j & 0x7fff_ffff) >= 0x4090_cc00
        && (((j as u32).wrapping_sub(0xc090_cc00) as i32 | i) != 0 || p_l <= z - p_h)
    {
        return s * TINY * TINY;
    }
    let i = j & 0x7fff_ffff;
    let mut k = (i >> 20) - 0x3ff;
    let mut n = 0;
    if i > 0x3fe0_0000 {
        n = j + (0x0010_0000 >> (k + 1));
        k = ((n & 0x7fff_ffff) >> 20) - 0x3ff;
        let t = from_words(n & !(0x000f_ffff >> k), 0);
        n = ((n & 0x000f_ffff) | 0x0010_0000) >> (20 - k);
        if j < 0 {
            n = -n;
        }
        p_h -= t;
    }
    let t = with_lo(p_l + p_h, 0);
    let u = t * LG2_H;
    let v = (p_l - (t - p_h)) * LG2 + t * LG2_L;
    let mut z = u + v;
    let w = v - (z - u);
    let t = z * z;
    let t1 = z - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));
    let r = (z * t1) / (t1 - 2.0) - (w + z * w);
    z = 1.0 - (r - z);
    let j = hi(z).wrapping_add(n << 20);
    if (j >> 20) <= 0 {
        z = scalbn(z, n);
    } else {
        z = with_hi(z, j);
    }
    s * z
}

/// fdlibm's s_scalbn.c: `x * 2**n` with a single rounding.
fn scalbn(x: f64, n: i32) -> f64 {
    const TWO54: f64 = f64::from_bits(0x4350_0000_0000_0000);
    const TWOM54: f64 = f64::from_bits(0x3C90_0000_0000_0000);
    let mut x = x;
    let mut hx = hi(x);
    let lx = lo(x);
    let mut k = (hx & 0x7ff0_0000) >> 20;
    if k == 0 {
        if (lx | (hx & 0x7fff_ffff) as u32) == 0 {
            return x;
        }
        x *= TWO54;
        hx = hi(x);
        k = ((hx & 0x7ff0_0000) >> 20) - 54;
        if n < -50000 {
            return TINY * x;
        }
    }
    if k == 0x7ff {
        return x + x;
    }
    k += n;
    if k > 0x7fe {
        return HUGE * HUGE.copysign(x);
    }
    if k > 0 {
        return with_hi(x, (hx & (0x800f_ffffu32 as i32)) | (k << 20));
    }
    if k <= -54 {
        return if n > 50000 {
            HUGE * HUGE.copysign(x)
        } else {
            TINY * TINY.copysign(x)
        };
    }
    k += 54;
    with_hi(x, (hx & (0x800f_ffffu32 as i32)) | (k << 20)) * TWOM54
}
