use super::*;

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn typed(preceding: &str, to_insert: &str) -> Option<(i32, String)> {
    actions(&units(preceding), &units(to_insert))
        .map(|action| (action.delete, String::from_utf16(&action.text).unwrap()))
}

fn replaced(text: &str) -> Option<(i32, String)> {
    Some((1, text.to_string()))
}

#[test]
fn an_initial_and_a_vowel_make_a_syllable() {
    assert_eq!(typed("ㄱ", "ㅏ"), replaced("가"));
    assert_eq!(typed("xㅎ", "ㅣ"), replaced("히"));
}

#[test]
fn a_consonant_after_a_syllable_is_its_final() {
    assert_eq!(typed("가", "ㄱ"), replaced("각"));
    assert_eq!(typed("가", "ㅄ"), replaced("값"));
}

#[test]
fn finals_combine() {
    assert_eq!(typed("각", "ㅅ"), replaced("갃"));
    assert_eq!(typed("갈", "ㄱ"), replaced("갉"));
    assert_eq!(typed("갈", "ㅎ"), replaced("갏"));
    assert_eq!(typed("ㄹ", "ㅂ"), replaced("ㄼ"));
    assert_eq!(typed("ㄴ", "ㅎ"), replaced("ㄶ"));
}

#[test]
fn a_vowel_takes_the_final_into_the_next_syllable() {
    assert_eq!(typed("각", "ㅏ"), replaced("가가"));
    assert_eq!(typed("갃", "ㅏ"), replaced("각사"));
    assert_eq!(typed("값", "ㅓ"), replaced("갑서"));
    assert_eq!(typed("갔", "ㅣ"), replaced("가씨"));
}

#[test]
fn vowels_combine() {
    assert_eq!(typed("고", "ㅏ"), replaced("과"));
    assert_eq!(typed("그", "ㅣ"), replaced("긔"));
    assert_eq!(typed("ㅗ", "ㅐ"), replaced("ㅙ"));
    assert_eq!(typed("ㅜ", "ㅣ"), replaced("ㅟ"));
    // Not once the syllable has a final.
    assert_eq!(typed("곡", "ㅐ"), replaced("고개"));
}

#[test]
fn anything_else_is_typed_as_it_is() {
    assert_eq!(typed("", "ㅏ"), None);
    assert_eq!(typed("ㄱ", ""), None);
    assert_eq!(typed("가", "_"), None);
    assert_eq!(typed("가", "ㄸ"), None);
    assert_eq!(typed("각", "ㄱ"), None);
    assert_eq!(typed("a", "ㅏ"), None);
    assert_eq!(typed("ㅏ", "ㅏ"), None);
    assert_eq!(typed("ㄱ", "ㄱ"), None);
    // Only the first unit of what is typed counts.
    assert_eq!(typed("ㄱ", "ㅏx"), replaced("가"));
}

#[test]
fn every_syllable_and_jamo_composes_within_hangul() {
    let jamo = (0x3131..=0x318E).chain([u16::from(b'_'), u16::from(b'a')]);
    let jamo: Vec<u16> = jamo.collect();
    for last in (0x3131..=0x318E).chain(FIRST_SYLLABLE..=LAST_SYLLABLE) {
        for &c in &jamo {
            if let Some(action) = actions(&[last], &[c]) {
                assert_eq!(action.delete, 1);
                assert!(
                    action
                        .text
                        .iter()
                        .all(|&u| (0x3131..=LAST_SYLLABLE).contains(&u))
                );
            }
        }
    }
}
