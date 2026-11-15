//! Presents CPU frames as buffers on an ASurfaceControl child of the SurfaceView's window,
//! each transaction tagged with its Choreographer vsync id (ASurfaceTransaction_setFrameTimeline),
//! so SurfaceFlinger records them as app SurfaceFrames, as it does for HWUI. A buffer posted
//! through ANativeWindow carries no vsync id: the platform's ANativeWindow_setFrameTimelineInfo
//! is not exported to apps. The symbols are resolved at run time because the native libraries
//! link against the API 26 stubs (`projectNativeMinSdk`), which predate them.
use std::{
    ffi::{c_char, c_void},
    sync::{Arc, Condvar, Mutex, OnceLock},
    time::{Duration, Instant},
};

/// What the CPU raster writes: one opaque frame of RGBA_8888 rows, `width` pixels apart.
pub(crate) struct Frame<'a> {
    pub pixels: &'a [u8],
    pub width: i32,
    pub height: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    fn union(self, other: Option<Rect>) -> Rect {
        let Some(o) = other else { return self };
        Rect {
            left: self.left.min(o.left),
            top: self.top.min(o.top),
            right: self.right.max(o.right),
            bottom: self.bottom.max(o.bottom),
        }
    }
}

#[repr(C)]
#[derive(Default)]
struct Desc {
    width: u32,
    height: u32,
    layers: u32,
    format: u32,
    usage: u64,
    stride: u32,
    rfu0: u32,
    rfu1: u64,
}

#[repr(C)]
struct PollFd {
    fd: i32,
    events: i16,
    revents: i16,
}

const FORMAT_RGBA_8888: u32 = 1;
const USAGE_CPU_WRITE_OFTEN: u64 = 3 << 4;
const USAGE_GPU_SAMPLED_IMAGE: u64 = 1 << 8;
const USAGE_COMPOSER_OVERLAY: u64 = 1 << 11;
/// One on screen, one queued, one being written: SurfaceFlinger holds at most two.
const BUFFERS: usize = 3;
const RTLD_NOW: i32 = 2;
const POLLIN: i16 = 1;
const VISIBILITY_SHOW: i8 = 1;
/// As opaque as the SurfaceView layer it replaces, whose default RGB_565 format has no alpha.
const TRANSPARENCY_OPAQUE: i8 = 2;

type Released = unsafe extern "C" fn(*mut c_void, i32);

/// The NDK entry points, with the signatures surface_control.h and hardware_buffer.h declare.
struct Api {
    create_from_window: unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void,
    release_control: unsafe extern "C" fn(*mut c_void),
    create: unsafe extern "C" fn() -> *mut c_void,
    delete: unsafe extern "C" fn(*mut c_void),
    apply: unsafe extern "C" fn(*mut c_void),
    set_buffer:
        unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, i32, *mut c_void, Released),
    set_visibility: unsafe extern "C" fn(*mut c_void, *mut c_void, i8),
    set_transparency: unsafe extern "C" fn(*mut c_void, *mut c_void, i8),
    set_damage: unsafe extern "C" fn(*mut c_void, *mut c_void, *const Rect, u32),
    set_timeline: unsafe extern "C" fn(*mut c_void, i64),
    reparent: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
    allocate: unsafe extern "C" fn(*const Desc, *mut *mut c_void) -> i32,
    release_buffer: unsafe extern "C" fn(*mut c_void),
    describe: unsafe extern "C" fn(*const c_void, *mut Desc),
    lock: unsafe extern "C" fn(*mut c_void, u64, i32, *const Rect, *mut *mut c_void) -> i32,
    unlock: unsafe extern "C" fn(*mut c_void, *mut i32) -> i32,
}

#[link(name = "dl")]
unsafe extern "C" {
    fn dlopen(name: *const c_char, flags: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
}

unsafe extern "C" {
    fn poll(fds: *mut PollFd, count: u64, timeout: i32) -> i32;
    fn close(fd: i32) -> i32;
}

/// Reads a resolved symbol as the function-pointer type `F`.
///
/// # Safety
/// `F` must be a function pointer with the symbol's declared signature.
unsafe fn cast<F: Copy>(found: *mut c_void) -> F {
    assert_eq!(size_of::<F>(), size_of::<*mut c_void>());
    // SAFETY: equal sizes (checked); the caller vouches the pointer is a function of type F.
    unsafe { std::mem::transmute_copy::<*mut c_void, F>(&found) }
}

fn api() -> Option<&'static Api> {
    static API: OnceLock<Option<Api>> = OnceLock::new();
    API.get_or_init(load).as_ref()
}

fn load() -> Option<Api> {
    // SAFETY: both names are NUL terminated; the system libraries stay loaded for the process.
    let (android, window) = unsafe {
        (
            dlopen(c"libandroid.so".as_ptr(), RTLD_NOW),
            dlopen(c"libnativewindow.so".as_ptr(), RTLD_NOW),
        )
    };
    if android.is_null() || window.is_null() {
        return None;
    }
    macro_rules! symbol {
        ($library:expr, $name:literal) => {{
            // SAFETY: the library handle is live and the name is NUL terminated.
            let found = unsafe { dlsym($library, concat!($name, "\0").as_ptr().cast()) };
            if found.is_null() {
                return None;
            }
            // SAFETY: the field's type is exactly the NDK header's declaration of this symbol.
            unsafe { cast(found) }
        }};
    }
    Some(Api {
        create_from_window: symbol!(android, "ASurfaceControl_createFromWindow"),
        release_control: symbol!(android, "ASurfaceControl_release"),
        create: symbol!(android, "ASurfaceTransaction_create"),
        delete: symbol!(android, "ASurfaceTransaction_delete"),
        apply: symbol!(android, "ASurfaceTransaction_apply"),
        set_buffer: symbol!(android, "ASurfaceTransaction_setBufferWithRelease"),
        set_visibility: symbol!(android, "ASurfaceTransaction_setVisibility"),
        set_transparency: symbol!(android, "ASurfaceTransaction_setBufferTransparency"),
        set_damage: symbol!(android, "ASurfaceTransaction_setDamageRegion"),
        set_timeline: symbol!(android, "ASurfaceTransaction_setFrameTimeline"),
        reparent: symbol!(android, "ASurfaceTransaction_reparent"),
        allocate: symbol!(window, "AHardwareBuffer_allocate"),
        release_buffer: symbol!(window, "AHardwareBuffer_release"),
        describe: symbol!(window, "AHardwareBuffer_describe"),
        lock: symbol!(window, "AHardwareBuffer_lock"),
        unlock: symbol!(window, "AHardwareBuffer_unlock"),
    })
}

/// Which buffers SurfaceFlinger has given back, each with the fence to wait on before writing.
struct Pool {
    free: Mutex<Slots>,
    released: Condvar,
}

struct Slots {
    fences: [Option<i32>; BUFFERS],
    /// The presenter is gone: a buffer given back now only has its fence closed.
    closed: bool,
}

impl Pool {
    fn slots(&self) -> std::sync::MutexGuard<'_, Slots> {
        self.free.lock().unwrap_or_else(|e| e.into_inner())
    }
}

struct Release {
    pool: Arc<Pool>,
    slot: usize,
}

/// ASurfaceTransaction_OnBufferRelease: invoked exactly once per buffer set, on any thread.
unsafe extern "C" fn released(context: *mut c_void, fence: i32) {
    // SAFETY: the context is the Box leaked for this one callback; the platform calls it once.
    let release = unsafe { Box::from_raw(context.cast::<Release>()) };
    let mut slots = release.pool.slots();
    if slots.closed {
        close_fence(fence);
    } else {
        slots.fences[release.slot] = Some(fence);
        release.pool.released.notify_all();
    }
}

pub(crate) struct SurfacePresenter {
    api: &'static Api,
    control: *mut c_void,
    buffers: [*mut c_void; BUFFERS],
    strides: [usize; BUFFERS],
    /// What each buffer missed since it was last written, so a partial frame can catch it up.
    stale: [Option<Rect>; BUFFERS],
    pool: Arc<Pool>,
    shown: bool,
}

// SAFETY: the presenter's handles are used only by the render thread that owns it.
unsafe impl Send for SurfacePresenter {}

impl SurfacePresenter {
    /// A child of `window` (an ANativeWindow the caller keeps alive for the presenter's life).
    pub(crate) fn new(window: *mut c_void, width: u16, height: u16) -> Option<Self> {
        let api = api()?;
        // SAFETY: the caller's window is live; the debug name is NUL terminated.
        let control = unsafe { (api.create_from_window)(window, c"aw-settings".as_ptr()) };
        if control.is_null() {
            return None;
        }
        let full = Rect {
            left: 0,
            top: 0,
            right: width.into(),
            bottom: height.into(),
        };
        let mut presenter = Self {
            api,
            control,
            buffers: [std::ptr::null_mut(); BUFFERS],
            strides: [0; BUFFERS],
            stale: [Some(full); BUFFERS],
            pool: Arc::new(Pool {
                free: Mutex::new(Slots {
                    fences: [Some(-1); BUFFERS],
                    closed: false,
                }),
                released: Condvar::new(),
            }),
            shown: false,
        };
        let request = Desc {
            width: width.into(),
            height: height.into(),
            layers: 1,
            format: FORMAT_RGBA_8888,
            usage: USAGE_CPU_WRITE_OFTEN | USAGE_GPU_SAMPLED_IMAGE | USAGE_COMPOSER_OVERLAY,
            ..Desc::default()
        };
        for slot in 0..BUFFERS {
            let mut described = Desc::default();
            // SAFETY: the descriptor and out-pointer are valid; a returned buffer is released on drop.
            unsafe {
                if (api.allocate)(&request, &mut presenter.buffers[slot]) != 0
                    || presenter.buffers[slot].is_null()
                {
                    return None;
                }
                (api.describe)(presenter.buffers[slot], &mut described);
            }
            presenter.strides[slot] = described.stride as usize;
        }
        Some(presenter)
    }

    /// Shows `frame` with `damage` repainted, as the frame Choreographer timed with `vsync`
    /// (negative for none). False when no buffer came back in time or the platform refused one.
    pub(crate) fn present(&mut self, frame: &Frame, damage: Rect, vsync: i64) -> bool {
        let start = Instant::now();
        let Some((slot, fence)) = self.acquire() else {
            return false;
        };
        if !wait(fence) {
            self.pool.slots().fences[slot] = Some(-1);
            return false;
        }
        let region = damage.union(self.stale[slot]);
        let waited = start.elapsed();
        let mut bits = std::ptr::null_mut();
        // SAFETY: the buffer is ours (SurfaceFlinger released it and its fence signalled).
        if unsafe {
            (self.api.lock)(
                self.buffers[slot],
                USAGE_CPU_WRITE_OFTEN,
                -1,
                &region,
                &mut bits,
            )
        } != 0
            || bits.is_null()
        {
            self.pool.slots().fences[slot] = Some(-1);
            return false;
        }
        let locked = start.elapsed();
        let (left, right) = (
            region.left.clamp(0, frame.width) as usize,
            region.right.clamp(0, frame.width) as usize,
        );
        for y in region.top.clamp(0, frame.height) as usize
            ..region.bottom.clamp(0, frame.height) as usize
        {
            let source = (y * frame.width as usize + left) * 4;
            let destination = (y * self.strides[slot] + left) * 4;
            if right > left && source + (right - left) * 4 <= frame.pixels.len() {
                // SAFETY: the row lies inside the locked stride*height buffer and inside `pixels`.
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        frame.pixels.as_ptr().add(source),
                        bits.cast::<u8>().add(destination),
                        (right - left) * 4,
                    );
                }
            }
        }
        let copied = start.elapsed();
        let mut written = -1;
        // SAFETY: exactly one unlock for the lock above; the fence it returns goes to the platform.
        unsafe { (self.api.unlock)(self.buffers[slot], &mut written) };
        for other in 0..BUFFERS {
            self.stale[other] = if other == slot {
                None
            } else {
                Some(damage.union(self.stale[other]))
            };
        }
        let release = Box::into_raw(Box::new(Release {
            pool: self.pool.clone(),
            slot,
        }))
        .cast::<c_void>();
        // SAFETY: the transaction is created, filled and applied here; the platform takes the
        // write fence, a reference to the buffer and the release context (freed in `released`).
        unsafe {
            let transaction = (self.api.create)();
            (self.api.set_buffer)(
                transaction,
                self.control,
                self.buffers[slot],
                written,
                release,
                released,
            );
            if !self.shown {
                (self.api.set_visibility)(transaction, self.control, VISIBILITY_SHOW);
                (self.api.set_transparency)(transaction, self.control, TRANSPARENCY_OPAQUE);
            }
            (self.api.set_damage)(transaction, self.control, &damage, 1);
            if vsync >= 0 {
                (self.api.set_timeline)(transaction, vsync);
            }
            (self.api.apply)(transaction);
            (self.api.delete)(transaction);
        }
        self.shown = true;
        if crate::render_window::profile_enabled() {
            // The same names as the ANativeWindow path's, plus the wait for a released buffer;
            // unlock_ns includes applying the transaction.
            crate::render_window::log(&format!(
                "buffer wait_ns={} lock_ns={} copy_ns={} unlock_ns={}",
                waited.as_nanos(),
                (locked - waited).as_nanos(),
                (copied - locked).as_nanos(),
                (start.elapsed() - copied).as_nanos()
            ));
        }
        true
    }

    fn acquire(&self) -> Option<(usize, i32)> {
        let mut slots = self.pool.slots();
        loop {
            if let Some(slot) = slots.fences.iter().position(Option::is_some) {
                return slots.fences[slot].take().map(|fence| (slot, fence));
            }
            let (next, waited) = self
                .pool
                .released
                .wait_timeout(slots, Duration::from_secs(1))
                .unwrap_or_else(|e| e.into_inner());
            slots = next;
            if waited.timed_out() && slots.fences.iter().all(Option::is_none) {
                return None;
            }
        }
    }
}

/// Waits for a release fence (if any) and closes it. False if it did not signal within a second.
fn wait(fence: i32) -> bool {
    if fence < 0 {
        return true;
    }
    let mut descriptor = PollFd {
        fd: fence,
        events: POLLIN,
        revents: 0,
    };
    // SAFETY: one valid pollfd for the duration of the call.
    let ready = unsafe { poll(&mut descriptor, 1, 1000) } == 1;
    close_fence(fence);
    ready
}

fn close_fence(fence: i32) {
    if fence >= 0 {
        // SAFETY: a release fence handed to us is ours to close, exactly once.
        unsafe { close(fence) };
    }
}

impl Drop for SurfacePresenter {
    fn drop(&mut self) {
        // SAFETY: detaching our layer makes SurfaceFlinger give back every buffer (each release
        // callback still owns its Pool reference); our own buffer and control references go last.
        unsafe {
            let transaction = (self.api.create)();
            (self.api.reparent)(transaction, self.control, std::ptr::null_mut());
            (self.api.apply)(transaction);
            (self.api.delete)(transaction);
            (self.api.release_control)(self.control);
            for buffer in self.buffers {
                if !buffer.is_null() {
                    (self.api.release_buffer)(buffer);
                }
            }
        }
        let mut slots = self.pool.slots();
        slots.closed = true;
        slots
            .fences
            .iter_mut()
            .filter_map(Option::take)
            .for_each(close_fence);
    }
}
