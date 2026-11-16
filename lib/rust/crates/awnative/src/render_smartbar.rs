//! Smartbar packets and the font-independent JVM differential-test boundary.
use crate::{
    handle,
    jni::{self, illegal},
};
use jni_sys::{JNIEnv, jboolean, jclass, jfloatArray, jintArray, jlong, jobjectArray};
use render::protocol::*;
use render::{
    smartbar::{self, Config},
    smartbar_scene::{Action, Smartbar},
};
use std::ptr;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_smartbar(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    config: jfloatArray,
    codes: jintArray,
    labels: jobjectArray,
) {
    // SAFETY: synchronized Kotlin owner keeps the handle and local arrays alive.
    unsafe {
        handle::with_locked::<crate::render::Session, _>(env, native, "renderer", (), |s, env| {
            let config = Config::decode(&env.read::<f32>(config, "config")?)
                .ok_or_else(|| illegal("invalid smartbar config"))?;
            let codes = env.read::<i32>(codes, "codes")?;
            let labels = crate::render::strings(env, labels)?;
            if codes.len() % BAR_ACTION_STRIDE as usize != 0
                || labels.len() < codes.len()
                || (labels.len() - codes.len()) % 2 != 0
                || codes.len() / BAR_ACTION_STRIDE as usize != 5 + config.dynamic
            {
                return Err(illegal("invalid smartbar actions"));
            }
            let actions = codes
                .as_chunks::<{ BAR_ACTION_STRIDE as usize }>()
                .0
                .iter()
                .enumerate()
                .map(|(i, c)| Action {
                    code: c[0],
                    enabled: c[BAR_FLAGS_OFFSET as usize] & BAR_ENABLED != 0,
                    lit: if c[BAR_FLAGS_OFFSET as usize] & BAR_DICTATING != 0 {
                        1
                    } else if c[BAR_FLAGS_OFFSET as usize] & BAR_READING != 0 {
                        2
                    } else {
                        0
                    },
                    level: (c[BAR_FLAGS_OFFSET as usize] >> BAR_LEVEL_SHIFT & BAR_LEVEL_MASK)
                        as f32
                        / 255.0,
                    warming: c[BAR_FLAGS_OFFSET as usize] & BAR_WARMING != 0,
                    finishing: c[BAR_FLAGS_OFFSET as usize] & BAR_FINISHING != 0,
                    icon: labels[i * 2].clone(),
                    label: labels[i * 2 + 1].clone(),
                    styles: Default::default(),
                })
                .collect();
            let mut bar = Smartbar::new(config, actions);
            bar.aux = labels[codes.len()..]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|a| (a[0].clone(), a[1].clone()))
                .collect();
            if let Some(old) = &s.scene.smartbar {
                bar.pressed = old.pressed.clone();
                bar.scroll = old.scroll;
                bar.phase = old.phase;
            }
            s.scene.smartbar = Some(bar);
            Ok(())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SmartbarRules_layout(
    env: *mut JNIEnv,
    _: jclass,
    config: jfloatArray,
    width: f32,
    intrinsic: jfloatArray,
) -> jfloatArray {
    // SAFETY: JVM local arrays are valid for this invocation and copied before use.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = Config::decode(&env.read::<f32>(config, "config")?)
                .ok_or_else(|| illegal("invalid smartbar config"))?;
            let intrinsic = env.read::<f32>(intrinsic, "intrinsic")?;
            if !width.is_finite()
                || width <= 0.0
                || intrinsic.iter().any(|v| !v.is_finite() || *v < 0.0)
            {
                return Err(illegal("invalid smartbar measurement"));
            }
            env.make(&smartbar::layout(&c, width, &intrinsic, 0.0).packed(&c))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SmartbarRules_ellipsis(
    env: *mut JNIEnv,
    _: jclass,
    advances: jfloatArray,
    width: f32,
) -> i32 {
    // SAFETY: JVM local array is copied within the invocation.
    unsafe {
        jni::run(env, 0, |env| {
            Ok(smartbar::ellipsis(&env.read::<f32>(advances, "advances")?, width) as i32)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SmartbarRules_roomy(
    env: *mut JNIEnv,
    _: jclass,
    config: jfloatArray,
) -> jboolean {
    // SAFETY: JVM local array is valid for this invocation and copied before use.
    unsafe {
        jni::run(env, 0, |env| {
            let c = Config::decode(&env.read::<f32>(config, "config")?)
                .ok_or_else(|| illegal("invalid smartbar config"))?;
            Ok(jboolean::from(c.roomy()))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SmartbarRules_toggleWidth(
    env: *mut JNIEnv,
    _: jclass,
    config: jfloatArray,
) -> f32 {
    // SAFETY: JVM local array is valid for this invocation and copied before use.
    unsafe {
        jni::run(env, 0.0, |env| {
            let c = Config::decode(&env.read::<f32>(config, "config")?)
                .ok_or_else(|| illegal("invalid smartbar config"))?;
            Ok(c.toggle_width())
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SmartbarRules_textScale(
    env: *mut JNIEnv,
    _: jclass,
    scale: f32,
    measured: f32,
    available: f32,
) -> f32 {
    // SAFETY: only primitives cross; env is valid for this invocation to raise the exception.
    unsafe {
        jni::run(env, 0.0, |_| {
            if !scale.is_finite() || scale <= 0.0 || measured.is_nan() || available.is_nan() {
                return Err(illegal("invalid text scale"));
            }
            Ok(smartbar::text_scale(scale, measured, available))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_candidateLongPress(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    pointer: i32,
) {
    // SAFETY: the synchronized Kotlin owner keeps the session handle alive for this call.
    unsafe {
        handle::with_locked::<crate::render::Session, _>(env, native, "renderer", (), |s, _| {
            s.touch.candidate_long_press(&mut s.scene, pointer);
            Ok(())
        });
    }
}

/// Deadlines use the MotionEvent uptime clock; the host schedules one wakeup, not idle frames.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_tick(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    now: jlong,
) -> jintArray {
    // SAFETY: the synchronized owner keeps the handle alive; the returned array belongs to the JVM.
    unsafe {
        handle::with_locked::<crate::render::Session, _>(
            env,
            native,
            "renderer",
            ptr::null_mut(),
            |s, env| {
                let actions: Vec<_> = s
                    .touch
                    .tick(&mut s.scene, now)
                    .into_iter()
                    .flat_map(|a| [a.0, a.1, a.2])
                    .collect();
                s.prepare_panel();
                let mut actions = actions;
                if s.scene
                    .search
                    .as_mut()
                    .and_then(|p| p.search.as_mut())
                    .is_some_and(|search| search.tick(now))
                {
                    actions.extend([render::protocol::PANEL_CHANGED, 0, 0]);
                }
                if s.scene.panel.as_mut().is_some_and(|p| p.media.tick(now)) {
                    actions.extend([render::protocol::PANEL_CHANGED, 0, 0]);
                }
                env.make(&actions)
            },
        )
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_nextDeadline(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
) -> jlong {
    // SAFETY: the synchronized owner keeps the handle alive.
    unsafe {
        handle::with_locked::<crate::render::Session, _>(env, native, "renderer", -1, |s, _| {
            Ok(s.touch
                .next_deadline()
                .into_iter()
                .chain(s.scene.panel.as_ref().and_then(|p| p.media.deadline()))
                .chain(
                    s.scene
                        .search
                        .as_ref()
                        .and_then(|p| p.search.as_ref())
                        .map(|search| search.deadline),
                )
                .min()
                .unwrap_or(-1))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_barSettings(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    delay: jlong,
    slots: jintArray,
) -> jintArray {
    // SAFETY: the synchronized owner keeps the handle and local array alive.
    unsafe {
        handle::with_locked::<crate::render::Session, _>(
            env,
            native,
            "renderer",
            ptr::null_mut(),
            |s, env| {
                s.touch.long_press_delay = delay;
                let slots = env.read::<i32>(slots, "changed slots")?;
                let actions: Vec<_> = s
                    .touch
                    .invalidate_bar(&mut s.scene, &slots)
                    .into_iter()
                    .flat_map(|a| [a.0, a.1, a.2])
                    .collect();
                s.prepare_panel();
                env.make(&actions)
            },
        )
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_candidateBindings(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    tokens: jintArray,
) -> jintArray {
    // SAFETY: synchronized owner keeps the handle and JVM arrays alive throughout this call.
    unsafe {
        handle::with_locked::<crate::render::Session, _>(
            env,
            native,
            "renderer",
            ptr::null_mut(),
            |s, env| {
                s.scene.candidate_tokens = env.read::<i32>(tokens, "candidate handles")?;
                env.make(&s.touch.candidate_tokens())
            },
        )
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_tooltipShown(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    pointer: i32,
    shown: jboolean,
) {
    // SAFETY: synchronized owner keeps the session alive for the platform tooltip acknowledgement.
    unsafe {
        handle::with_locked::<crate::render::Session, _>(env, native, "renderer", (), |s, _| {
            s.touch.tooltip_shown(pointer, shown != 0);
            Ok(())
        });
    }
}
