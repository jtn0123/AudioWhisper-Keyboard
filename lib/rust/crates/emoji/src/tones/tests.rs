use super::*;
use crate::utf16;

#[test]
fn the_first_listed_tone_wins() {
    assert_eq!(tones(&utf16("😀")), (0, 0));
    assert_eq!(tones(&utf16("👋🏽")), (3, 0));
    assert_eq!(tones(&utf16("🧑🏿‍🦰")), (5, 1));
    // Two skin tones: the first in the enum's order, not in the text's.
    assert_eq!(tones(&utf16("🧑🏿‍🤝‍🧑🏻")), (1, 0));
    assert_eq!(tones(&utf16("👨‍🦳👨‍🦲")), (0, 3));
}

#[test]
fn a_nul_is_default_and_a_lone_half_is_not_a_tone() {
    assert_eq!(tones(&utf16("👋🏽\0🦰")), (0, 0));
    let tone = utf16("🏽");
    assert_eq!(tones(&tone[..1]), (0, 0));
    assert_eq!(tones(&tone[1..]), (0, 0));
    assert_eq!(tones(&[tone[1], tone[0]]), (0, 0));
    assert_eq!(tones(&[]), (0, 0));
}
