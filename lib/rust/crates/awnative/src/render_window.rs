//! Android-only NDK ownership. No window pointer escapes its render thread after acquisition.
use crate::jni::{Result, illegal};
use jni_sys::{JNIEnv, jobject};
use keys::Rect;
use render::{raster::Raster, scene::Scene};
use std::{
    ffi::c_void,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
    time::Instant,
};

use crate::surface_bounds::Rect as ARect;
#[repr(C)]
struct Buffer {
    width: i32,
    height: i32,
    stride: i32,
    format: i32,
    bits: *mut c_void,
    reserved: [u32; 6],
}

#[link(name = "android")]
unsafe extern "C" {
    fn ANativeWindow_fromSurface(env: *mut JNIEnv, surface: jobject) -> *mut c_void;
    fn ANativeWindow_release(window: *mut c_void);
    fn ANativeWindow_setBuffersGeometry(
        window: *mut c_void,
        width: i32,
        height: i32,
        format: i32,
    ) -> i32;
    fn ANativeWindow_lock(window: *mut c_void, buffer: *mut Buffer, dirty: *mut ARect) -> i32;
    fn ANativeWindow_unlockAndPost(window: *mut c_void) -> i32;
}

#[derive(Debug)]
struct Window {
    raw: usize,
    generation: u64,
}
static NEXT_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
static LIVE_WINDOWS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
impl Drop for Window {
    fn drop(&mut self) {
        // SAFETY: fromSurface acquired this non-null reference; only this owner releases it, after all posts finish.
        unsafe {
            ANativeWindow_release(self.raw as *mut c_void);
        }
        let live = LIVE_WINDOWS.fetch_sub(1, std::sync::atomic::Ordering::Relaxed) - 1;
        log(&format!(
            "window released generation={} live={live}",
            self.generation
        ));
    }
}

/// The damage as whole buffer pixels, clipped to the scene.
fn dirty(scene: &Scene, damage: Rect) -> ARect {
    ARect {
        left: damage.left.floor().max(0.0) as i32,
        top: damage.top.floor().max(0.0) as i32,
        right: damage.right.ceil().min(scene.width as f32) as i32,
        bottom: damage.bottom.ceil().min(scene.height as f32) as i32,
    }
}

impl Window {
    fn post(&self, scene: &Scene, pixels: &[u8], damage: Rect) -> bool {
        let window = self.raw as *mut c_void;
        let mut dirty = dirty(scene, damage);
        let mut buffer = Buffer {
            width: 0,
            height: 0,
            stride: 0,
            format: 0,
            bits: std::ptr::null_mut(),
            reserved: [0; 6],
        };
        let lock_start = monotonic_ns();
        // SAFETY: the owned window is live; NDK writes these initialized stack values and returns an exclusively locked buffer.
        if unsafe { ANativeWindow_lock(window, &mut buffer, &mut dirty) } != 0 {
            return false;
        }
        let locked = monotonic_ns();
        let valid = buffer.width == scene.width as i32
            && buffer.height == scene.height as i32
            && buffer.stride >= buffer.width
            && buffer.format == 1
            && !buffer.bits.is_null();
        if valid {
            let left = dirty.left.clamp(0, buffer.width) as usize;
            let right = dirty.right.clamp(left as i32, buffer.width) as usize;
            for y in dirty.top.clamp(0, buffer.height)..dirty.bottom.clamp(0, buffer.height) {
                let source = (y as usize * scene.width as usize + left) * 4;
                let destination = (y as usize * buffer.stride as usize + left) * 4;
                // SAFETY: both rows are within validated sizes; NDK owns a buffer of stride*height RGBA pixels until unlock.
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        pixels.as_ptr().add(source),
                        buffer.bits.cast::<u8>().add(destination),
                        (right - left) * 4,
                    );
                }
            }
        }
        let copied = monotonic_ns();
        // SAFETY: exactly one unlock for the successful lock above, even on unexpected dimensions/format.
        let posted = unsafe { ANativeWindow_unlockAndPost(window) } == 0;
        if profile_enabled() {
            log(&format!(
                "buffer lock_ns={} copy_ns={} unlock_ns={}",
                locked - lock_start,
                copied - locked,
                monotonic_ns() - copied
            ));
        }
        valid && posted
    }
}

/// The next scene to draw, with its touch time, Choreographer frame time and vsync id.
#[derive(Debug, Default)]
struct Pending {
    next: Option<(Scene, i64, i64, i64)>,
    stop: bool,
}

#[derive(Debug)]
pub struct Worker {
    shared: Arc<(Mutex<Pending>, Condvar)>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new(
        env: *mut JNIEnv,
        surface: jobject,
        fonts: Vec<render::text::FontBlob>,
        width: u16,
        height: u16,
        hybrid: bool,
        timeline: bool,
    ) -> Result<Self> {
        if surface.is_null() || width == 0 || height == 0 {
            return Err(illegal("surface without scene dimensions"));
        }
        // SAFETY: JNI Surface is a live local reference; fromSurface acquires an independent native reference.
        let window = unsafe { ANativeWindow_fromSurface(env, surface) };
        if window.is_null() {
            return Err(illegal("could not acquire surface"));
        }
        let generation = NEXT_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let window = Window {
            raw: window as usize,
            generation,
        };
        let live = LIVE_WINDOWS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        log(&format!(
            "window acquired generation={generation} live={live}"
        ));
        // SAFETY: window is owned and no render thread is using it yet. Format 1 is WINDOW_FORMAT_RGBA_8888.
        if unsafe {
            ANativeWindow_setBuffersGeometry(
                window.raw as *mut c_void,
                width as i32,
                height as i32,
                1,
            )
        } != 0
        {
            return Err(illegal("could not configure native buffer"));
        }
        let shared = Arc::new((Mutex::new(Pending::default()), Condvar::new()));
        let worker_shared = shared.clone();
        let thread = thread::Builder::new()
            .name("aw-render".into())
            .spawn(move || {
                run(
                    window,
                    worker_shared,
                    fonts,
                    width,
                    height,
                    hybrid,
                    timeline,
                );
            })
            .map_err(|_| illegal("could not start render thread"))?;
        Ok(Self {
            shared,
            thread: Some(thread),
        })
    }

    pub fn submit(&self, scene: Scene, touch: i64, frame: i64, vsync: i64) {
        let (lock, ready) = &*self.shared;
        lock.lock().unwrap_or_else(|e| e.into_inner()).next = Some((scene, touch, frame, vsync));
        ready.notify_one();
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let (lock, ready) = &*self.shared;
        lock.lock().unwrap_or_else(|e| e.into_inner()).stop = true;
        ready.notify_one();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run(
    window: Window,
    shared: Arc<(Mutex<Pending>, Condvar)>,
    fonts: Vec<render::text::FontBlob>,
    width: u16,
    height: u16,
    hybrid: bool,
    timeline: bool,
) {
    let profiling = profile_enabled();
    crate::render_profile::heap(
        2,
        fonts.iter().map(|f| f.len()).sum(),
        width as usize * height as usize * 4,
    );
    let initialization = profiling.then(Instant::now);
    let gpu = if hybrid {
        window.gpu(width, height, &fonts)
    } else {
        None
    };
    let mut engine =
        gpu.unwrap_or_else(|| Engine::Cpu(Box::new(Raster::shared(width, height, fonts))));
    crate::render_profile::heap(3, 0, width as usize * height as usize * 4);
    // Declared after `window` (a parameter), so it is dropped, and detached, first.
    let mut presenter = match &engine {
        Engine::Cpu(_) if timeline => {
            let presenter = crate::render_surface::SurfacePresenter::new(
                window.raw as *mut c_void,
                width,
                height,
            );
            log(&format!(
                "surface_control={}",
                u8::from(presenter.is_some())
            ));
            presenter
        }
        _ => None,
    };
    let mut previous = Scene::default();
    if let Some(raster) = engine.cpu() {
        raster.set_profiling(profiling);
    }
    if let Some(initialization) = initialization {
        log(&format!(
            "profile_init_ns={}",
            initialization.elapsed().as_nanos()
        ));
    }
    let (lock, ready) = &*shared;
    loop {
        let mut pending = lock.lock().unwrap_or_else(|e| e.into_inner());
        while pending.next.is_none() && !pending.stop {
            pending = ready.wait(pending).unwrap_or_else(|e| e.into_inner());
        }
        if pending.stop {
            break;
        }
        let Some((scene, touch, frame, vsync)) = pending.next.take() else {
            continue;
        };
        drop(pending);
        let regions = previous.damage_regions(&scene);
        let Some(damage) = regions.iter().copied().reduce(render::scene::union) else {
            continue;
        };
        let worker_ns = monotonic_ns();
        let _trace = crate::render_profile::Span::new(c"aw.worker");
        let start = Instant::now();
        let cpu_start = thread_cpu_ns();
        let (draw, draw_cpu, gpu_wait, post_ns) = match &mut engine {
            Engine::Cpu(raster) => {
                if raster.draw_regions(&scene, &regions).is_err() {
                    log("draw_error");
                    break;
                }
                let draw = start.elapsed().as_nanos();
                let draw_cpu = thread_cpu_ns() - cpu_start;
                let post_start = Instant::now();
                let posted = match &mut presenter {
                    Some(presenter) => presenter.present(
                        &crate::render_surface::Frame {
                            pixels: raster.pixels(),
                            width: scene.width.into(),
                            height: scene.height.into(),
                        },
                        dirty(&scene, damage),
                        vsync,
                    ),
                    None => window.post(&scene, raster.pixels(), damage),
                };
                if !posted {
                    log("post_error");
                    break;
                }
                (draw, draw_cpu, 0, post_start.elapsed().as_nanos())
            }
            #[cfg(feature = "hybrid")]
            Engine::Hybrid(gpu) => match gpu.draw(&scene) {
                Ok((draw, gpu_wait)) => (draw, thread_cpu_ns() - cpu_start, gpu_wait, 0),
                Err(error) => {
                    log(&format!("hybrid_draw_error={error}"));
                    break;
                }
            },
        };
        let posted = monotonic_ns();
        if previous.width == 0 {
            crate::render_profile::heap(4, 0, width as usize * height as usize * 4);
        }
        if profiling && let Some(raster) = engine.cpu() {
            let p = raster.profile();
            let area = ((damage.right.ceil().min(scene.width as f32)
                - damage.left.floor().max(0.0))
            .max(0.0)
                * (damage.bottom.ceil().min(scene.height as f32) - damage.top.floor().max(0.0))
                    .max(0.0)) as u64;
            let paint = render::profile::damage_pixels(&regions, scene.width, scene.height);
            let region_count = regions.len();
            let full = paint == u64::from(scene.width) * u64::from(scene.height);
            let press = scene.pressed.iter().any(|i| !previous.pressed.contains(i));
            let release = previous.pressed.iter().any(|i| !scene.pressed.contains(i));
            let recreate = previous.width == 0;
            let shift = previous
                .keys
                .iter()
                .zip(&scene.keys)
                .any(|(a, b)| a.label != b.label && a.label.eq_ignore_ascii_case(&b.label));
            log(&format!(
                "profile draw_ns={draw} cpu_ns={draw_cpu} buffer_post_ns={post_ns} press={} release={} recreate={} full={} area_px={area} paint_px={paint} regions={region_count} shift={} popup_change={} popup_open={} candidates={} encode_ns={} raster_ns={} shape_ns={} glyph_ns={} alloc_ns={} alloc_sites={} layout_misses={} labels={} cold_glyph_ns={} color_glyph_ns={} glyph_runs={} cold_runs={} color_runs={}",
                u8::from(press),
                u8::from(release),
                u8::from(recreate),
                u8::from(full),
                u8::from(shift),
                u8::from(previous.popup != scene.popup),
                u8::from(scene.popup.is_some()),
                u8::from(previous.candidates != scene.candidates),
                p.encode_ns,
                p.raster_ns,
                p.shape_ns,
                p.glyph_ns,
                p.alloc_ns,
                p.alloc_sites,
                p.layout_misses,
                p.labels,
                p.cold_glyph_ns,
                p.color_glyph_ns,
                p.glyph_runs,
                p.cold_runs,
                p.color_runs
            ));
        }
        // Only numeric timing data; never scene labels, candidates, coordinates or action codes.
        log(&format!(
            "frame draw_ns={draw} post_ns={} touch_ns={touch} choreographer_ns={frame} vsync_id={vsync} worker_ns={worker_ns} gpu_wait_ns={gpu_wait}",
            posted
        ));
        previous = scene;
    }
}

pub(crate) fn profile_enabled() -> bool {
    #[link(name = "log")]
    unsafe extern "C" {
        fn __android_log_is_loggable(
            priority: i32,
            tag: *const std::ffi::c_char,
            default: i32,
        ) -> i32;
    }
    // SAFETY: the static tag is NUL terminated; this function retains no pointers.
    unsafe { __android_log_is_loggable(3, c"aw-profile".as_ptr(), 4) != 0 }
}

enum Engine {
    Cpu(Box<Raster>),
    #[cfg(feature = "hybrid")]
    Hybrid(Box<render::hybrid::Hybrid>),
}

impl Engine {
    /// The CPU renderer, which draws into memory this side presents; None for the GPU's.
    fn cpu(&mut self) -> Option<&mut Raster> {
        match self {
            Self::Cpu(raster) => Some(raster),
            #[cfg(feature = "hybrid")]
            Self::Hybrid(_) => None,
        }
    }
}

impl Window {
    /// The GPU renderer for this window, if the `aw-hybrid` switch asked for it and it starts.
    #[cfg(feature = "hybrid")]
    fn gpu(&self, width: u16, height: u16, fonts: &[render::text::FontBlob]) -> Option<Engine> {
        match self.hybrid(width, height, fonts.to_vec()) {
            Ok(gpu) => {
                log(&format!("hybrid adapter={}", gpu.adapter));
                Some(Engine::Hybrid(Box::new(gpu)))
            }
            Err(error) => {
                log(&format!("hybrid_init_error={error}"));
                None
            }
        }
    }

    /// Without the `hybrid` feature there is no GPU renderer: the switch is ignored and the CPU
    /// one draws, so a phone with `aw-hybrid` set behaves as one without.
    #[cfg(not(feature = "hybrid"))]
    fn gpu(&self, _: u16, _: u16, _: &[render::text::FontBlob]) -> Option<Engine> {
        log("hybrid_unavailable");
        None
    }

    #[cfg(feature = "hybrid")]
    fn hybrid(
        &self,
        width: u16,
        height: u16,
        fonts: Vec<render::text::FontBlob>,
    ) -> std::result::Result<render::hybrid::Hybrid, String> {
        use raw_window_handle::{AndroidDisplayHandle, AndroidNdkWindowHandle};
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let pointer =
            std::ptr::NonNull::new(self.raw as *mut c_void).ok_or("null native window")?;
        // SAFETY: this thread exclusively owns the acquired ANativeWindow. The GPU and its
        // Surface are dropped before this Window, including on initialization failures.
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(AndroidDisplayHandle::new().into()),
                raw_window_handle: AndroidNdkWindowHandle::new(pointer).into(),
            })
        }
        .map_err(|_| "Vulkan surface unavailable")?;
        render::hybrid::Hybrid::new(&instance, surface, width, height, fonts)
    }
}

fn monotonic_ns() -> i64 {
    // CLOCK_MONOTONIC is 1 on Android.
    clock_ns(1)
}

/// The CPU time this thread has used: where it is far below the wall time of the same work, the
/// thread was waiting to run, not working.
fn thread_cpu_ns() -> i64 {
    // CLOCK_THREAD_CPUTIME_ID is 3 on Android.
    clock_ns(3)
}

fn clock_ns(clock: i32) -> i64 {
    #[repr(C)]
    struct Timespec {
        seconds: i64,
        nanos: i64,
    }
    unsafe extern "C" {
        fn clock_gettime(clock: i32, time: *mut Timespec) -> i32;
    }
    let mut time = Timespec {
        seconds: 0,
        nanos: 0,
    };
    // SAFETY: both callers pass a clock Android defines; the pointer is writable for one timespec.
    if unsafe { clock_gettime(clock, &mut time) } == 0 {
        time.seconds * 1_000_000_000 + time.nanos
    } else {
        0
    }
}

pub(crate) fn log(message: &str) {
    #[link(name = "log")]
    unsafe extern "C" {
        fn __android_log_write(
            priority: i32,
            tag: *const std::ffi::c_char,
            text: *const std::ffi::c_char,
        ) -> i32;
    }
    if let Ok(message) = std::ffi::CString::new(message) {
        // SAFETY: both strings are NUL terminated and valid for the synchronous Android log call.
        unsafe {
            __android_log_write(3, c"aw-render".as_ptr(), message.as_ptr());
        }
    }
}
