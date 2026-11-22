//! Smartbar packets, and the smartbar's font-independent rules for `SmartbarNative`, whose
//! generated entry points call them.
use crate::{
    jni::{Env, Result, illegal},
    render::{session, texts},
};
use jni_sys::{jobject, jvalue};
use render::protocol::*;
use render::{
    smartbar::{self, Config},
    smartbar_fit::{self, TextFit},
    smartbar_scene::{Action, Smartbar},
};
use std::cell::Cell;

pub fn smartbar(native: i64, config: &[f32], codes: &[i32], labels: &[Vec<u16>]) -> Result<()> {
    session(native, |s| {
        let config = Config::decode(config).ok_or_else(|| illegal("invalid smartbar config"))?;
        let labels = texts(labels);
        if !codes.len().is_multiple_of(BAR_ACTION_STRIDE as usize)
            || labels.len() < codes.len()
            || !(labels.len() - codes.len()).is_multiple_of(2)
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
                level: (c[BAR_FLAGS_OFFSET as usize] >> BAR_LEVEL_SHIFT & BAR_LEVEL_MASK) as f32
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
    })
}
/// A smartbar config from its packed floats.
fn config(config: &[f32]) -> Result<Config> {
    Config::decode(config).ok_or_else(|| illegal("invalid smartbar config"))
}

/// Where the smartbar's slots go (`smartbar::layout`), packed.
pub fn layout(config: &[f32], width: f32, intrinsic: &[f32]) -> Result<Vec<f32>> {
    let c = self::config(config)?;
    if !width.is_finite() || width <= 0.0 || intrinsic.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(illegal("invalid smartbar measurement"));
    }
    Ok(smartbar::layout(&c, width, intrinsic, 0.0).packed(&c))
}

/// How many characters of `advances` fit in `width` with an ellipsis (`smartbar::ellipsis`).
pub fn ellipsis(advances: &[f32], width: f32) -> i32 {
    smartbar::ellipsis(advances, width) as i32
}

/// Whether words show in Gboard's row (`Config::roomy`).
pub fn roomy(config: &[f32]) -> Result<bool> {
    Ok(self::config(config)?.roomy())
}

/// The arrow's slot width in pixels (`Config::toggle_width`).
pub fn toggle_width(config: &[f32]) -> Result<f32> {
    Ok(self::config(config)?.toggle_width())
}

/// A text fit as it crosses: its squeeze, then its scale.
fn packed_fit(fit: TextFit) -> Vec<f32> {
    vec![fit.squeeze, fit.scale]
}

/// One fitting step (`smartbar_fit::fit_step`).
pub fn fit_step(squeeze: f32, scale: f32, measured: f32, available: f32) -> Result<Vec<f32>> {
    let fit = |v: f32| v.is_finite() && v > 0.0;
    if !fit(squeeze) || !fit(scale) || measured.is_nan() || available.is_nan() {
        return Err(illegal("invalid text fit"));
    }
    let next = smartbar_fit::fit_step(TextFit { squeeze, scale }, measured, available);
    Ok(packed_fit(next))
}

/// How a word draws in `available` (`smartbar_fit::fit_text`), asking `measure`, a
/// `SmartbarRules.TextMeasure`, for its width at each font scale tried. The first failure, a
/// Kotlin exception or a width of NaN, ends the fit: later measures are not asked.
pub fn fit_text(env: &Env, available: f32, measure: jobject) -> Result<Vec<f32>> {
    if available.is_nan() {
        return Err(illegal("invalid text fit"));
    }
    let method = env.method(measure, "measure", (c"measure", c"(F)F"))?;
    let failed = Cell::new(None);
    let fit = smartbar_fit::fit_text(available, |scale| {
        let width = match failed.take() {
            Some(error) => Err(error),
            None => env.call_float(method, &[jvalue { f: scale }]),
        };
        let width = width.and_then(|width| match width.is_nan() {
            true => Err(illegal("invalid text fit")),
            false => Ok(width),
        });
        width.unwrap_or_else(|error| {
            failed.set(Some(error));
            available
        })
    });
    match failed.take() {
        Some(error) => Err(error),
        None => Ok(packed_fit(fit)),
    }
}
pub fn candidate_long_press(native: i64, pointer: i32) -> Result<()> {
    session(native, |s| {
        s.touch.candidate_long_press(&mut s.scene, pointer);
        Ok(())
    })
}

/// Deadlines use the MotionEvent uptime clock; the host schedules one wakeup, not idle frames.
pub fn tick(native: i64, now: i64) -> Result<Vec<i32>> {
    session(native, |s| {
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
        Ok(actions)
    })
}
pub fn next_deadline(native: i64) -> Result<i64> {
    session(native, |s| {
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
pub fn bar_settings(native: i64, delay: i64, slots: &[i32]) -> Result<Vec<i32>> {
    session(native, |s| {
        s.touch.long_press_delay = delay;
        let actions: Vec<_> = s
            .touch
            .invalidate_bar(&mut s.scene, slots)
            .into_iter()
            .flat_map(|a| [a.0, a.1, a.2])
            .collect();
        s.prepare_panel();
        Ok(actions)
    })
}

pub fn candidate_bindings(native: i64, tokens: &[i32]) -> Result<Vec<i32>> {
    session(native, |s| {
        s.scene.candidate_tokens = tokens.to_vec();
        Ok(s.touch.candidate_tokens())
    })
}

pub fn tooltip_shown(native: i64, pointer: i32, shown: bool) -> Result<()> {
    session(native, |s| {
        s.touch.tooltip_shown(pointer, shown);
        Ok(())
    })
}
