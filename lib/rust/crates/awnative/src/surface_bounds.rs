//! What part of a CPU frame the settings presenter may copy into a locked hardware buffer.
//!
//! The copy in render_surface.rs is unsafe: it writes rows straight into memory the platform
//! lent us. This is the arithmetic it relies on, kept pure so the host can test it. A frame is
//! copied only when it is exactly the buffer's size, as the keyboard's ANativeWindow path insists
//! (render_window.rs `post`); the settings scene can change size (onSizeChanged) before
//! surfaceChanged makes buffers of the new size, and such a frame is skipped.

/// A pixel rectangle, as the NDK's ARect: left and top inclusive, right and bottom exclusive.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    /// The smallest rectangle covering both.
    pub(crate) fn union(self, other: Option<Rect>) -> Rect {
        let Some(o) = other else { return self };
        Rect {
            left: self.left.min(o.left),
            top: self.top.min(o.top),
            right: self.right.max(o.right),
            bottom: self.bottom.max(o.bottom),
        }
    }

    /// This rectangle inside `width`×`height`; empty (never inverted) when it lies outside.
    pub(crate) fn clamped(self, width: i32, height: i32) -> Rect {
        let left = self.left.clamp(0, width);
        let top = self.top.clamp(0, height);
        Rect {
            left,
            top,
            right: self.right.clamp(left, width),
            bottom: self.bottom.clamp(top, height),
        }
    }
}

/// A hardware buffer as AHardwareBuffer_describe reports it: pixels, and pixels per row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Allocated {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

/// One row of the copy: byte offsets into the frame and into the buffer, and its length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Span {
    pub source: usize,
    pub destination: usize,
    pub bytes: usize,
}

/// A checked copy: the region, inside both the frame and the buffer, and how rows map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Plan {
    region: Rect,
    width: usize,
    stride: usize,
}

const BYTES_PER_PIXEL: usize = 4;

impl Plan {
    /// The region to lock and repaint.
    pub(crate) fn region(&self) -> Rect {
        self.region
    }

    /// Each row of the region, top to bottom.
    pub(crate) fn spans(&self) -> impl Iterator<Item = Span> + '_ {
        let (left, right) = (self.region.left as usize, self.region.right as usize);
        (self.region.top as usize..self.region.bottom as usize).map(move |y| Span {
            source: (y * self.width + left) * BYTES_PER_PIXEL,
            destination: (y * self.stride + left) * BYTES_PER_PIXEL,
            bytes: (right - left) * BYTES_PER_PIXEL,
        })
    }
}

/// How to copy `damage` of a `width`×`height` RGBA frame held in `pixels` bytes into `buffer`.
/// None, so the frame is skipped, unless the frame is exactly the buffer's size, the buffer's
/// rows are at least that wide and `pixels` holds the whole frame. The damage is clamped to it.
pub(crate) fn plan(
    buffer: Allocated,
    width: i32,
    height: i32,
    pixels: usize,
    damage: Rect,
) -> Option<Plan> {
    let (Ok(frame_width), Ok(frame_height)) = (u32::try_from(width), u32::try_from(height)) else {
        return None;
    };
    let needed = (frame_width as usize)
        .checked_mul(frame_height as usize)?
        .checked_mul(BYTES_PER_PIXEL)?;
    let fits = frame_width == buffer.width
        && frame_height == buffer.height
        && buffer.stride >= buffer.width
        && pixels >= needed;
    fits.then(|| Plan {
        region: damage.clamped(width, height),
        width: frame_width as usize,
        stride: buffer.stride as usize,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUFFER: Allocated = Allocated {
        width: 8,
        height: 6,
        stride: 16,
    };

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
        Rect {
            left,
            top,
            right,
            bottom,
        }
    }

    fn whole(width: i32, height: i32) -> usize {
        (width * height) as usize * BYTES_PER_PIXEL
    }

    #[test]
    fn a_frame_the_buffers_size_is_copied_where_it_changed() {
        let plan = plan(BUFFER, 8, 6, whole(8, 6), rect(2, 1, 5, 3)).unwrap();
        assert_eq!(plan.region(), rect(2, 1, 5, 3));
        let spans: Vec<Span> = plan.spans().collect();
        assert_eq!(
            spans,
            [1, 2].map(|y| Span {
                source: (y * 8 + 2) * 4,
                destination: (y * 16 + 2) * 4,
                bytes: 12,
            })
        );
    }

    #[test]
    fn a_frame_of_another_size_is_skipped() {
        // The report's case: narrower and taller passes a check of the source alone, then
        // writes rows past the buffer's end.
        assert_eq!(plan(BUFFER, 4, 12, whole(4, 12), rect(0, 0, 4, 12)), None);
        assert_eq!(plan(BUFFER, 16, 3, whole(16, 3), rect(0, 0, 16, 3)), None);
        assert_eq!(plan(BUFFER, 8, 7, whole(8, 7), rect(0, 0, 8, 7)), None);
        assert_eq!(plan(BUFFER, 9, 6, whole(9, 6), rect(0, 0, 9, 6)), None);
        assert_eq!(plan(BUFFER, -8, 6, whole(8, 6), rect(0, 0, 8, 6)), None);
    }

    #[test]
    fn a_buffer_whose_rows_are_narrower_than_it_is_wide_is_refused() {
        let short = Allocated {
            stride: 7,
            ..BUFFER
        };
        assert_eq!(plan(short, 8, 6, whole(8, 6), rect(0, 0, 8, 6)), None);
    }

    #[test]
    fn pixels_that_do_not_cover_the_frame_are_refused() {
        assert_eq!(plan(BUFFER, 8, 6, whole(8, 6) - 1, rect(0, 0, 1, 1)), None);
    }

    #[test]
    fn damage_is_clamped_to_the_buffer_before_it_is_locked() {
        let plan = plan(BUFFER, 8, 6, whole(8, 6), rect(-3, -2, 20, 40)).unwrap();
        assert_eq!(plan.region(), rect(0, 0, 8, 6));
        let outside = super::plan(BUFFER, 8, 6, whole(8, 6), rect(10, 9, 30, 20)).unwrap();
        assert_eq!(outside.region(), rect(8, 6, 8, 6));
        assert_eq!(outside.spans().count(), 0);
        let inverted = super::plan(BUFFER, 8, 6, whole(8, 6), rect(5, 4, 2, 1)).unwrap();
        assert_eq!(inverted.region(), rect(5, 4, 5, 4));
    }

    #[test]
    fn no_row_ever_ends_outside_the_frame_or_the_buffer() {
        let sizes = [(1, 1, 1), (3, 2, 3), (8, 6, 16), (5, 7, 9)];
        let edges = [-4, -1, 0, 1, 2, 4, 5, 7, 8, 12];
        for (bw, bh, stride) in sizes {
            let buffer = Allocated {
                width: bw,
                height: bh,
                stride,
            };
            let locked = ((bh - 1) * stride + bw) as usize * BYTES_PER_PIXEL;
            for (fw, fh) in [(bw, bh), (bw / 2 + 1, bh * 2), (bw * 2, bh / 2 + 1)] {
                let pixels = whole(fw as i32, fh as i32);
                let damages = edges.iter().flat_map(|&l| edges.map(|t| (l, t)));
                for (l, t) in damages {
                    for (r, b) in edges.iter().zip(edges.iter().rev()) {
                        let damage = rect(l, t, *r, *b);
                        let Some(plan) = plan(buffer, fw as i32, fh as i32, pixels, damage) else {
                            assert!((fw, fh) != (bw, bh), "a matching frame must be copied");
                            continue;
                        };
                        for span in plan.spans() {
                            assert!(span.source + span.bytes <= pixels);
                            assert!(span.destination + span.bytes <= locked);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn union_covers_both_rectangles() {
        let a = rect(1, 2, 3, 4);
        assert_eq!(a.union(None), a);
        assert_eq!(a.union(Some(rect(0, 3, 2, 9))), rect(0, 2, 3, 9));
    }
}
