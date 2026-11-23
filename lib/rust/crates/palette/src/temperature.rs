// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// TemperatureCache (the complement and analogous colours used by the Content
// and Fidelity schemes) and DislikeAnalyzer.
//
// The Kotlin keys temperatures by `Hct`, whose equality is by ARGB; a
// colour's temperature depends only on its ARGB, so storing the temperature
// beside each colour gives the same lookups. Sorting is stable in both.

use crate::color::{self, lab_from_argb, sanitize_degrees, sanitize_degrees_int};
use crate::hct::Hct;
use crate::jmath;

/// A colour with its raw temperature.
#[derive(Clone, Copy)]
struct Warmth {
    hct: Hct,
    temp: f64,
}

/// `TemperatureCache` for one input colour.
pub(crate) struct TemperatureCache {
    input: Warmth,
    by_hue: Vec<Warmth>,
    coldest: f64,
    warmest: Warmth,
    coldest_hct: Hct,
}

impl TemperatureCache {
    /// Builds `hctsByHue`, `tempsByHct` and `hctsByTemp` eagerly.
    pub(crate) fn new(input: Hct) -> TemperatureCache {
        let mut by_hue = Vec::with_capacity(361);
        let mut hue = 0.0;
        while hue <= 360.0 {
            let hct = Hct::from(hue, input.chroma, input.tone);
            by_hue.push(Warmth {
                hct,
                temp: raw_temperature(&hct),
            });
            hue += 1.0;
        }
        let input = Warmth {
            hct: input,
            temp: raw_temperature(&input),
        };
        let mut by_temp = by_hue.clone();
        by_temp.push(input);
        by_temp.sort_by(|a, b| a.temp.total_cmp(&b.temp));
        let coldest = by_temp[0];
        let warmest = by_temp[by_temp.len() - 1];
        TemperatureCache {
            input,
            by_hue,
            coldest: coldest.temp,
            warmest,
            coldest_hct: coldest.hct,
        }
    }

    fn relative_temperature(&self, w: Warmth) -> f64 {
        let range = self.warmest.temp - self.coldest;
        let difference_from_coldest = w.temp - self.coldest;
        if range == 0.0 {
            0.5
        } else {
            difference_from_coldest / range
        }
    }

    fn at_hue(&self, hue: f64) -> Warmth {
        self.by_hue[jmath::to_int(jmath::rint(hue)) as usize]
    }

    /// `TemperatureCache.complement`.
    pub(crate) fn complement(&self) -> Hct {
        let coldest_hue = self.coldest_hct.hue;
        let coldest_temp = self.coldest;
        let warmest_hue = self.warmest.hct.hue;
        let warmest_temp = self.warmest.temp;
        let range = warmest_temp - coldest_temp;
        let start_is_coldest_to_warmest = is_between(self.input.hct.hue, coldest_hue, warmest_hue);
        let start_hue = if start_is_coldest_to_warmest {
            warmest_hue
        } else {
            coldest_hue
        };
        let end_hue = if start_is_coldest_to_warmest {
            coldest_hue
        } else {
            warmest_hue
        };
        let direction_of_rotation = 1.0;
        let mut smallest_error = 1000.0;
        let mut answer = self.at_hue(self.input.hct.hue).hct;
        let complement_relative_temp = 1.0 - self.relative_temperature(self.input);
        let mut hue_addend = 0.0;
        while hue_addend <= 360.0 {
            let hue = sanitize_degrees(start_hue + direction_of_rotation * hue_addend);
            if !is_between(hue, start_hue, end_hue) {
                hue_addend += 1.0;
                continue;
            }
            let possible_answer = self.at_hue(hue);
            let relative_temp = (possible_answer.temp - coldest_temp) / range;
            let error = (complement_relative_temp - relative_temp).abs();
            if error < smallest_error {
                smallest_error = error;
                answer = possible_answer.hct;
            }
            hue_addend += 1.0;
        }
        answer
    }

    /// `TemperatureCache.getAnalogousColors(count, divisions)`.
    pub(crate) fn analogous_colors(&self, count: i32, divisions: i32) -> Vec<Hct> {
        let start_hue = jmath::to_int(jmath::rint(self.input.hct.hue));
        let start = self.by_hue[start_hue as usize];
        let mut last_temp = self.relative_temperature(start);
        let mut all_colors = vec![start];
        let mut absolute_total_temp_delta = 0.0;
        for i in 0..=359 {
            let w = self.by_hue[sanitize_degrees_int(start_hue + i) as usize];
            let temp = self.relative_temperature(w);
            absolute_total_temp_delta += (temp - last_temp).abs();
            last_temp = temp;
        }
        let mut hue_addend = 1;
        let temp_step = absolute_total_temp_delta / f64::from(divisions);
        let mut total_temp_delta = 0.0;
        last_temp = self.relative_temperature(start);
        let divisions = divisions as usize;
        while all_colors.len() < divisions {
            let w = self.by_hue[sanitize_degrees_int(start_hue + hue_addend) as usize];
            let temp = self.relative_temperature(w);
            total_temp_delta += (temp - last_temp).abs();
            let mut desired = all_colors.len() as f64 * temp_step;
            let mut index_satisfied = total_temp_delta >= desired;
            let mut index_addend = 1;
            while index_satisfied && all_colors.len() < divisions {
                all_colors.push(w);
                desired = (all_colors.len() + index_addend) as f64 * temp_step;
                index_satisfied = total_temp_delta >= desired;
                index_addend += 1;
            }
            last_temp = temp;
            hue_addend += 1;
            if hue_addend > 360 {
                while all_colors.len() < divisions {
                    all_colors.push(w);
                }
                break;
            }
        }
        let size = all_colors.len() as i32;
        let pick = |mut index: i32| {
            while index < 0 {
                index += size;
            }
            if index >= size {
                index %= size;
            }
            all_colors[index as usize].hct
        };
        let mut answers = vec![self.input.hct];
        let ccw_count = jmath::to_int(((f64::from(count) - 1.0) / 2.0).floor());
        for i in 1..ccw_count + 1 {
            answers.insert(0, pick(-i));
        }
        let cw_count = count - ccw_count - 1;
        for i in 1..cw_count + 1 {
            answers.push(pick(i));
        }
        answers
    }
}

/// `TemperatureCache.rawTemperature`.
fn raw_temperature(color: &Hct) -> f64 {
    let lab = lab_from_argb(color.argb);
    let hue = sanitize_degrees(color::to_degrees(jmath::atan2(lab[2], lab[1])));
    let chroma = jmath::hypot(lab[1], lab[2]);
    -0.5 + (0.02
        * jmath::pow(chroma, 1.07)
        * jmath::cos(color::to_radians(sanitize_degrees(hue - 50.0))))
}

/// `TemperatureCache.isBetween`.
fn is_between(angle: f64, a: f64, b: f64) -> bool {
    if a < b {
        a <= angle && angle <= b
    } else {
        a <= angle || angle <= b
    }
}

/// `DislikeAnalyzer.fixIfDisliked`.
pub(crate) fn fix_if_disliked(hct: Hct) -> Hct {
    let hue_passes = (90.0..=111.0).contains(&jmath::rint(hct.hue));
    let chroma_passes = jmath::rint(hct.chroma) > 16.0;
    let tone_passes = jmath::rint(hct.tone) < 65.0;
    if hue_passes && chroma_passes && tone_passes {
        Hct::from(hct.hue, hct.chroma, 70.0)
    } else {
        hct
    }
}
