//! Opt-in numeric attribution. Allocation counters cover our explicit allocation sites,
//! not allocations inside Parley/Vello or the platform allocator.
use std::time::Instant;

#[derive(Clone, Copy, Debug, Default)]
pub struct Profile {
    pub encode_ns: u128,
    pub raster_ns: u128,
    pub shape_ns: u128,
    pub glyph_ns: u128,
    pub cold_glyph_ns: u128,
    pub color_glyph_ns: u128,
    pub alloc_ns: u128,
    pub alloc_sites: u32,
    pub layout_misses: u32,
    pub labels: u32,
    pub glyph_runs: u32,
    pub cold_runs: u32,
    pub color_runs: u32,
}

pub(crate) fn elapsed(start: Option<Instant>) -> u128 {
    start.map_or(0, |start| start.elapsed().as_nanos())
}

/// Exact union of outward-rounded, surface-clamped damage pixels; profiling only.
pub fn damage_pixels(regions: &[keys::Rect], width: u16, height: u16) -> u64 {
    let mut intervals = Vec::with_capacity(regions.len());
    let mut pixels = 0;
    for y in 0..i32::from(height) {
        intervals.clear();
        for r in regions {
            if y >= r.top.floor() as i32 && y < r.bottom.ceil() as i32 {
                let left = (r.left.floor() as i32).clamp(0, i32::from(width));
                let right = (r.right.ceil() as i32).clamp(left, i32::from(width));
                intervals.push((left, right));
            }
        }
        intervals.sort_unstable();
        let mut end = 0;
        for &(left, right) in &intervals {
            pixels += (right - left.max(end)).max(0) as u64;
            end = end.max(right);
        }
    }
    pixels
}

impl Profile {
    pub(crate) fn accumulate(&mut self, other: Self) {
        macro_rules! add {
            ($($field:ident),*) => { $(self.$field += other.$field;)* };
        }
        add!(
            encode_ns,
            raster_ns,
            shape_ns,
            glyph_ns,
            cold_glyph_ns,
            color_glyph_ns,
            alloc_ns,
            alloc_sites,
            layout_misses,
            labels,
            glyph_runs,
            cold_runs,
            color_runs
        );
    }
}
