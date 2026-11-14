//! Numeric Android trace/heap receipts. Never accept UI strings or editable data.
#[cfg(target_os = "android")]
mod android {
    use std::ffi::{CStr, c_char};
    #[repr(C)]
    struct Mallinfo {
        values: [usize; 10],
    }
    // SAFETY: these declarations match the bionic malloc and public NDK trace headers.
    unsafe extern "C" {
        fn mallinfo() -> Mallinfo;
    }
    #[link(name = "android")]
    // SAFETY: NDK declarations use only C strings and no retained JNI references.
    unsafe extern "C" {
        fn ATrace_beginSection(section: *const c_char);
        fn ATrace_endSection();
    }
    pub struct Span;
    impl Span {
        pub fn new(name: &'static CStr) -> Self {
            // SAFETY: NDK consumes a static NUL-terminated literal synchronously.
            unsafe {
                ATrace_beginSection(name.as_ptr());
            }
            Self
        }
    }
    impl Drop for Span {
        fn drop(&mut self) {
            // SAFETY: this guard balances the section on the same calling thread.
            unsafe {
                ATrace_endSection();
            }
        }
    }
    pub fn heap(stage: u32, font_bytes: usize, pixel_bytes: usize) {
        if !super::super::render_window::profile_enabled() {
            return;
        }
        // SAFETY: bionic mallinfo returns its documented ten size_t fields by value.
        let allocated = unsafe { mallinfo().values[7] };
        super::super::render_window::log(&format!(
            "heap stage={stage} allocated_bytes={allocated} font_bytes={font_bytes} pixel_bytes={pixel_bytes}"
        ));
    }
}
#[cfg(target_os = "android")]
pub use android::{Span, heap};
