//! Re-ranks swipe guesses with the words before them, so the swipe for "to" after "me" gives
//! "too" once the user has typed "me too". Each guess's cost (how badly the swipe fits it, in nats;
//! lower is better) goes down by what the context says for it:
//!  - the user's habit: how often they typed it after that word;
//!  - the dictionary's n-gram, which the bundled one has for at most three next words a word.
//!
//! A missing pair says little, so neither ever makes a guess worse.
//!
//! The bonuses are small on purpose: together at most [`MAX_BONUS`]. The classifier's own costs
//! are close together (common words get the same frequency, see
//! [`crate::swipe_words::frequency_of`]): words with the same path ("to", "too") differ by about
//! 0.07, and a neighbouring key's word is within 0.1-0.5 of a clean swipe ("an" even beats "am" by
//! 0.14). So context settles those, but not a swipe that clearly traces a different word ("you"
//! over "thou": 1.4-2.2).
//!
//! Was Kotlin's `GlideRanker`, which still asks the learned words and the dictionary what they know
//! of each guess ([`Context`]): this turns that into bonuses and the order.

use crate::java::{compare_f32, min_f32};
use crate::raw::NOT_A_PROBABILITY;

/// Per ln(1 + uses): 0.31 for one use, 0.49 for two, 1.0 (the most) from about ten.
pub const HABIT_WEIGHT: f32 = 0.45;
pub const MAX_HABIT_BONUS: f32 = 1.0;

/// For a dictionary n-gram, plus a little per step it rises above the word alone.
pub const NGRAM_BONUS: f32 = 0.25;
pub const NGRAM_BONUS_PER_STEP: f32 = 0.01;
pub const MAX_NGRAM_BONUS: f32 = 0.6;

pub const MAX_BONUS: f32 = MAX_HABIT_BONUS + MAX_NGRAM_BONUS;

/// What the words before a guess say for it: how many times the user typed it right after them,
/// and the dictionary's n-gram probability for it there with its probability alone (0-255, or
/// [`NOT_A_PROBABILITY`] for none; `probability` matters only with an n-gram).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    pub pair_count: i32,
    pub ngram: i32,
    pub probability: i32,
}

/// How much `context` favours its guess, in nats.
pub fn bonus(context: Context) -> f32 {
    habit_bonus(context.pair_count) + dictionary_bonus(context.ngram, context.probability)
}

fn habit_bonus(count: i32) -> f32 {
    if count <= 0 {
        return 0.0;
    }
    // Kotlin's ln of a Float: Math.log of it as a Double, back to a Float.
    let uses = 1.0 + count as f32;
    let ln = f64::from(uses).ln() as f32;
    min_f32(HABIT_WEIGHT * ln, MAX_HABIT_BONUS)
}

/// An n-gram is the word's own 0-255 probability raised towards 255 by how often it follows the
/// words before it. That it exists at all is most of the evidence, given how few there are.
fn dictionary_bonus(ngram: i32, probability: i32) -> f32 {
    if ngram == NOT_A_PROBABILITY {
        return 0.0;
    }
    let rise = ngram.wrapping_sub(probability.max(0)).max(0);
    min_f32(
        NGRAM_BONUS + rise as f32 * NGRAM_BONUS_PER_STEP,
        MAX_NGRAM_BONUS,
    )
}

/// The guesses with these `costs` and `contexts`, best first in that context, as their indexes:
/// by cost less bonus, ordered as Kotlin orders floats ([`compare_f32`]), and ties in the
/// classifier's order. `None` if `contexts` doesn't have one per cost.
pub fn rank(costs: &[f32], contexts: &[Context]) -> Option<Vec<usize>> {
    if costs.len() != contexts.len() {
        return None;
    }
    let scores: Vec<f32> = costs
        .iter()
        .zip(contexts)
        .map(|(&cost, &context)| cost - bonus(context))
        .collect();
    let mut order: Vec<usize> = (0..scores.len()).collect();
    // Stable, as Kotlin's sortedBy is.
    order.sort_by(|&a, &b| compare_f32(scores[a], scores[b]));
    Some(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn habit(pair_count: i32) -> Context {
        Context {
            pair_count,
            ngram: NOT_A_PROBABILITY,
            probability: NOT_A_PROBABILITY,
        }
    }

    fn ngram(ngram: i32, probability: i32) -> Context {
        Context {
            pair_count: 0,
            ngram,
            probability,
        }
    }

    #[test]
    fn a_habit_counts_more_with_use_up_to_a_cap() {
        assert_eq!(bonus(habit(0)), 0.0);
        assert_eq!(bonus(habit(-5)), 0.0);
        assert!((bonus(habit(1)) - 0.312).abs() < 0.001);
        assert!((bonus(habit(2)) - 0.494).abs() < 0.001);
        assert_eq!(bonus(habit(10)), 1.0);
        assert_eq!(bonus(habit(i32::MAX)), 1.0);
    }

    #[test]
    fn an_ngram_counts_more_the_more_it_rises() {
        assert_eq!(bonus(ngram(NOT_A_PROBABILITY, 100)), 0.0);
        // "thank you": 5 steps above "you" alone.
        assert!((bonus(ngram(176, 171)) - 0.30).abs() < 0.001);
        assert_eq!(bonus(ngram(100, 200)), 0.25);
        assert_eq!(bonus(ngram(200, NOT_A_PROBABILITY)), 0.6);
        // A probability below 0 counts as 0, and an n-gram below 0 (but not "none") still counts.
        assert!((bonus(ngram(10, -20)) - 0.35).abs() < 0.001);
        assert_eq!(bonus(ngram(-5, 100)), 0.25);
        let both = Context {
            pair_count: 1000,
            ngram: 200,
            probability: 159,
        };
        assert_eq!(bonus(both), MAX_BONUS);
    }

    #[test]
    fn ranks_by_cost_less_bonus_keeping_ties() {
        let none = habit(0);
        // "to", "too", "top" after "me", where the user has typed "me too" once.
        let order = rank(&[3.28, 3.35, 3.54], &[none, habit(1), none]);
        assert_eq!(order, Some(vec![1, 0, 2]));
        assert_eq!(rank(&[2.0, 1.0, 2.0], &[none; 3]), Some(vec![1, 0, 2]));
        let odd = [f32::NAN, f32::INFINITY, 0.0, -0.0, f32::NEG_INFINITY];
        assert_eq!(rank(&odd, &[none; 5]), Some(vec![4, 3, 2, 1, 0]));
        assert_eq!(rank(&[1.0], &[]), None);
    }

    #[test]
    fn keeps_ties_in_order_however_many_guesses() {
        // Past 20 elements an unstable sort would reorder these.
        let costs: Vec<f32> = (0..50)
            .map(|i| if i % 3 == 0 { 1.0 } else { 2.0 })
            .collect();
        let expected: Vec<usize> = (0..50)
            .step_by(3)
            .chain((0..50).filter(|i| i % 3 != 0))
            .collect();
        let none = habit(0);
        assert_eq!(rank(&costs, &[none; 50]), Some(expected));
    }
}
