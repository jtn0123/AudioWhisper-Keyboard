//! Words typed without their apostrophe. AOSP's decoder already offers "don't" for "dont" and its
//! whitelist turns "im" into "I'm", but where the letters spell a word too ("its", "were",
//! "well", "ill", "lets") the dictionary cannot tell which was meant: a valid word is never
//! corrected, and a whitelisted one always is ("he is ill" became "he is I'll"). The language
//! model can, from the words before it, so where it clearly prefers one spelling it decides; where
//! it doesn't, or it lacks either word, AOSP's decision stands. See docs/autocorrect-results.md.

use chars::equals_ignore_case;

/// How much likelier, in nats, the model must find one spelling than the other to decide between
/// them: chosen on the scoreboard's tuning sentences (docs/autocorrect-results.md).
pub const MARGIN: f32 = 1.5;

const APOSTROPHE: u16 = b'\'' as u16;
const RIGHT_QUOTE: u16 = 0x2019;

/// What the model says about a word and its spelling with an apostrophe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The apostrophe was meant: correct to it.
    Restore,
    /// The word was meant as typed: leave it, whitelisted or not.
    Keep,
    /// The model can't tell, or doesn't know both: AOSP decides.
    Undecided,
}

/// Whether `candidate` is `typed` with one or more apostrophes put in ("dont" -> "don't",
/// "Ill" -> "I'll").
pub fn restores(typed: &[u16], candidate: &[u16]) -> bool {
    let is_apostrophe = |unit: &u16| *unit == APOSTROPHE || *unit == RIGHT_QUOTE;
    if !candidate.iter().any(is_apostrophe) {
        return false;
    }
    let letters: Vec<u16> = candidate
        .iter()
        .copied()
        .filter(|u| !is_apostrophe(u))
        .collect();
    equals_ignore_case(&letters, typed)
}

/// The verdict from `odds`: how much likelier, in nats, the model finds the apostrophe's spelling
/// than the typed one after the words before it, or none if it lacks either.
pub fn verdict(odds: Option<f32>) -> Verdict {
    match odds {
        Some(odds) if odds >= MARGIN => Verdict::Restore,
        Some(odds) if odds <= -MARGIN => Verdict::Keep,
        _ => Verdict::Undecided,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn a_spelling_with_apostrophes_put_in_restores_the_word() {
        for (typed, candidate) in [("dont", "don't"), ("Ill", "I'll"), ("its", "it\u{2019}s")] {
            assert!(
                restores(&utf16(typed), &utf16(candidate)),
                "{typed} {candidate}"
            );
        }
        for (typed, candidate) in [
            ("its", "its"),
            ("its", "it'd"),
            ("were", "where"),
            ("ill", "I'm"),
        ] {
            assert!(
                !restores(&utf16(typed), &utf16(candidate)),
                "{typed} {candidate}"
            );
        }
    }

    #[test]
    fn the_model_decides_only_when_it_clearly_prefers_one() {
        assert_eq!(verdict(Some(MARGIN)), Verdict::Restore);
        assert_eq!(verdict(Some(-MARGIN)), Verdict::Keep);
        assert_eq!(verdict(Some(MARGIN * 0.9)), Verdict::Undecided);
        assert_eq!(verdict(Some(-MARGIN * 0.9)), Verdict::Undecided);
        assert_eq!(verdict(None), Verdict::Undecided);
    }
}
