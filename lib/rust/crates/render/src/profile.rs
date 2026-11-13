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
