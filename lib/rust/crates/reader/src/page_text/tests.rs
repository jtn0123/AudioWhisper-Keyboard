use super::*;

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn edges(top: i32, bottom: i32) -> Edges {
    Edges { top, bottom }
}

/// Blocks 100 px tall, one under the other from `first_top`.
fn page(count: i32, first_top: i32) -> Vec<Edges> {
    (0..count)
        .map(|it| edges(first_top + it * 100, first_top + it * 100 + 100))
        .collect()
}

fn joined(pieces: &[(&str, i32, i32, bool)]) -> Vec<(String, Edges)> {
    let texts: Vec<Vec<u16>> = pieces.iter().map(|it| utf16(it.0)).collect();
    let pieces: Vec<Piece> = (pieces.iter().zip(&texts))
        .map(|(&(_, top, bottom, scrolls), text)| Piece {
            text,
            edges: edges(top, bottom),
            scrolls,
        })
        .collect();
    (blocks(&pieces).into_iter())
        .map(|it| (String::from_utf16_lossy(&it.text), it.edges))
        .collect()
}

fn find_in(texts: &[&str], first_top: i32, selected: &str, visible: Edges) -> Option<Start> {
    let units: Vec<Vec<u16>> = texts.iter().map(|it| utf16(it)).collect();
    let places = page(texts.len() as i32, first_top);
    let blocks: Vec<(&[u16], Edges)> = units.iter().map(|it| &it[..]).zip(places).collect();
    find(&blocks, &utf16(selected), visible)
}

fn resume(blocks: &[&str], read: &[&str]) -> usize {
    let blocks: Vec<Vec<u16>> = blocks.iter().map(|it| utf16(it)).collect();
    let read: Vec<Vec<u16>> = read.iter().map(|it| utf16(it)).collect();
    let blocks: Vec<&[u16]> = blocks.iter().map(|it| &it[..]).collect();
    let read: Vec<&[u16]> = read.iter().map(|it| &it[..]).collect();
    resume_after(&blocks, &read)
}

fn say(sentences: &[&str], max: usize) -> Vec<String> {
    let units: Vec<Vec<u16>> = sentences.iter().map(|it| utf16(it)).collect();
    let sentences: Vec<&[u16]> = units.iter().map(|it| &it[..]).collect();
    let max = NonZeroUsize::new(max).expect("a positive max");
    (utterances(&sentences, max).iter())
        .map(|it| String::from_utf16_lossy(it))
        .collect()
}

const SCREEN: Edges = Edges {
    top: 0,
    bottom: 1000,
};

#[test]
fn reading_starts_at_the_first_block_on_screen_even_if_its_top_is_cut_off() {
    let blocks = page(3, -150);
    assert_eq!(first_on_screen(&blocks, SCREEN), Some(1));
    assert_eq!(first_on_screen(&blocks, edges(2000, 3000)), None);
    assert!(!all_shown(&blocks, SCREEN));
    assert!(all_shown(&blocks[1..], SCREEN));
    assert!(all_shown(&[], SCREEN));
}

#[test]
fn a_block_scrolled_out_of_sight_is_not_on_screen() {
    // A web page squashes what's scrolled away to no height at the edge it went past.
    let blocks = [edges(210, 210), edges(210, 400), edges(2339, 2339)];
    assert_eq!(first_on_screen(&blocks, edges(0, 2339)), Some(1));
    assert!(!edges(400, 300).is_shown(SCREEN));
    assert!(!edges(1000, 1100).is_shown(SCREEN));
    assert!(!edges(-100, 0).is_shown(SCREEN));
    assert!(edges(999, 1100).is_shown(SCREEN));
}

#[test]
fn a_paragraph_with_a_link_in_it_is_read_as_one() {
    let blocks = joined(&[
        ("First has a ", 0, 50, false),
        ("link", 0, 50, false),
        (" in it.", 0, 100, false),
        ("Next paragraph", 120, 170, false),
        ("bold", 120, 170, false),
        (", then more.", 120, 170, false),
        ("Quote", 180, 200, false),
        ("\u{201D}", 180, 200, false),
    ]);
    let texts: Vec<&str> = blocks.iter().map(|it| &it.0[..]).collect();
    let expected = [
        "First has a link in it.",
        "Next paragraph",
        "bold, then more.",
        "Quote\u{201D}",
    ];
    assert_eq!(texts, expected);
    assert_eq!(blocks[0].1, edges(0, 100));
}

#[test]
fn bullets_and_pieces_with_no_words_are_dropped() {
    let blocks = joined(&[
        ("• ", 0, 50, false),
        ("Item one", 0, 50, false),
        ("◦▪\u{00A0}", 60, 100, false),
        ("Item two", 60, 100, false),
        ("|", 110, 130, false),
        ("", 140, 150, false),
        ("·", 160, 170, false),
        ("3", 180, 190, false),
    ]);
    let texts: Vec<&str> = blocks.iter().map(|it| &it.0[..]).collect();
    assert_eq!(texts, ["Item one", "Item two", "3"]);
    assert!(joined(&[]).is_empty());
}

#[test]
fn a_joined_block_spans_its_pieces_that_are_in_sight() {
    let fixed_then_list = joined(&[("Cut ", 300, 300, false), ("off", 300, 380, true)]);
    assert_eq!(
        fixed_then_list.len(),
        2,
        "a fixed bar and the list are never joined"
    );
    let block = joined(&[("Cut ", 300, 300, false), ("off", 300, 380, false)]);
    assert_eq!(block, [("Cut off".to_owned(), edges(300, 380))]);
    let none_shown = joined(&[("Gone ", 50, 50, true), ("away", 40, 10, true)]);
    assert_eq!(none_shown, [("Gone away".to_owned(), edges(50, 50))]);
}

#[test]
fn a_joined_block_knows_its_first_piece() {
    let texts = [utf16("• "), utf16("a "), utf16("b"), utf16("c")];
    let pieces: Vec<Piece> = (texts.iter())
        .map(|text| Piece {
            text,
            edges: edges(0, 10),
            scrolls: false,
        })
        .collect();
    let firsts: Vec<usize> = blocks(&pieces).iter().map(|it| it.first).collect();
    assert_eq!(firsts, [0, 3]);
}

#[test]
fn a_selection_starts_at_its_word_in_the_block_that_holds_it() {
    let texts = ["Hello there.", "The quick brown fox jumps.", "Bye."];
    let found = find_in(&texts, 0, "brown", SCREEN);
    assert_eq!(
        found,
        Some(Start {
            block: 1,
            offset: 10
        })
    );
}

#[test]
fn a_selection_matches_whatever_the_case_and_spacing() {
    let found = find_in(&["Line one\n  and   Two"], 0, "AND \t two", SCREEN);
    assert_eq!(
        found,
        Some(Start {
            block: 0,
            offset: 11
        })
    );
}

#[test]
fn a_selection_across_paragraphs_is_found_by_its_first_line() {
    let texts = ["First para.", "Second para."];
    for selected in ["para.\nSecond", "\r\n  para.  \rSecond", " \n para.\n"] {
        let found = find_in(&texts, 0, selected, SCREEN);
        assert_eq!(
            found,
            Some(Start {
                block: 0,
                offset: 6
            })
        );
    }
}

#[test]
fn the_same_words_on_screen_win_over_a_copy_off_it() {
    let found = find_in(&["ok", "intro", "ok"], -100, "ok", edges(150, 1000));
    assert_eq!(
        found,
        Some(Start {
            block: 2,
            offset: 0
        })
    );
    let off_screen = find_in(&["intro", "ok", "ok"], 2000, "ok", SCREEN);
    assert_eq!(
        off_screen,
        Some(Start {
            block: 1,
            offset: 0
        })
    );
}

#[test]
fn text_that_isnt_there_is_not_found() {
    assert_eq!(find_in(&["abc"], 0, "xyz", SCREEN), None);
    assert_eq!(find_in(&["abc"], 0, "  \n ", SCREEN), None);
    assert_eq!(find_in(&["abc"], 0, "", SCREEN), None);
    assert_eq!(find_in(&["ab"], 0, "abc", SCREEN), None);
    assert_eq!(find_in(&[""], 0, "a", SCREEN), None);
}

#[test]
fn a_selection_is_lowered_whole_and_the_page_a_unit_at_a_time() {
    // As in Kotlin: İ lowers to i and a combining dot in the selection, to plain i on the page.
    assert_eq!(find_in(&["İstanbul"], 0, "İstanbul", SCREEN), None);
    let found = find_in(&["visit İstanbul"], 0, "STANBUL", SCREEN);
    assert_eq!(
        found,
        Some(Start {
            block: 0,
            offset: 7
        })
    );
}

#[test]
fn after_a_scroll_reading_carries_on_past_the_last_blocks_read() {
    assert_eq!(resume(&["b", "c", "d", "e", "f"], &["a", "b", "c", "d"]), 3);
}

#[test]
fn a_repeated_label_doesnt_send_reading_back() {
    // "Copy" follows every message; the run of the last three blocks read pins the place.
    let read = ["one", "Copy", "two", "Copy"];
    let after = ["one", "Copy", "two", "Copy", "three", "Copy"];
    assert_eq!(resume(&after, &read), 4);
}

#[test]
fn a_shorter_run_is_matched_when_the_longer_one_is_gone() {
    assert_eq!(resume(&["x", "c", "d", "e"], &["a", "b", "c", "d"]), 3);
    assert_eq!(resume(&["d", "e"], &["a", "b", "c", "d"]), 1);
}

#[test]
fn a_scroll_past_everything_read_carries_on_from_the_first_new_block() {
    assert_eq!(resume(&["x", "y"], &["a", "b"]), 0);
    assert_eq!(resume(&["b", "x"], &["z", "b", "q"]), 1);
    let read: Vec<String> = (0..60).map(|it| it.to_string()).collect();
    let read: Vec<&str> = read.iter().map(|it| &it[..]).collect();
    // Only the last 50 read count as recent: "5" was read before them, "12" among them.
    assert_eq!(resume(&["12", "13", "5"], &read[..59]), 2);
    assert_eq!(resume(&["12", "x"], &read[..59]), 1);
    assert_eq!(resume(&["12", "13"], &read[..59]), 2);
}

#[test]
fn nothing_new_after_a_scroll_is_the_end() {
    assert_eq!(resume(&["a", "b"], &["a", "b"]), 2);
    assert_eq!(resume(&["b", "a"], &["a", "b", "c"]), 2);
    assert_eq!(resume(&[], &["a"]), 0);
    assert_eq!(resume(&["a"], &[]), 0);
}

#[test]
fn text_is_cut_into_sentences_the_speech_engine_takes() {
    let sentences = ["First one. ", "Second one! ", "Third?"];
    assert_eq!(
        say(&sentences, MAX_UTTERANCE),
        ["First one. Second one! Third?"]
    );
    assert_eq!(say(&sentences, 12), ["First one.", "Second one!", "Third?"]);
    assert_eq!(say(&sentences, 23), ["First one. Second one!", "Third?"]);
}

#[test]
fn a_sentence_too_long_for_the_engine_is_cut_at_spaces() {
    let long: Vec<String> = (0..10).map(|it| format!("word{it}")).collect();
    let long = long.join(" ");
    let pieces = say(&[&long], 20);
    assert!(pieces.iter().all(|it| it.len() <= 20), "{pieces:?}");
    assert_eq!(pieces.join(" "), long);
    // A space just past the limit still cuts there.
    assert_eq!(say(&["abcde fgh"], 5), ["abcde", "fgh"]);
    assert_eq!(say(&["  ab cd ef  "], 5), ["ab cd", "ef"]);
}

#[test]
fn a_sentence_with_nowhere_to_cut_is_cut_at_the_limit() {
    assert_eq!(say(&["abcdefgh"], 3), ["abc", "def", "gh"]);
    assert_eq!(say(&[" abcdefgh"], 3), ["abc", "def", "gh"]);
    // A unit at a time, cut at the space after the first.
    assert_eq!(
        say(&["a bcdefgh"], 1),
        ["a", "b", "c", "d", "e", "f", "g", "h"]
    );
    // As in Kotlin, a cut can fall between the halves of an emoji.
    let cut = say(&["a\u{1F600}b"], 2);
    assert_eq!(cut.len(), 2);
}

#[test]
fn a_long_sentence_is_said_apart_from_those_around_it() {
    let sentences = ["Hi. ", "abcdefgh ", "Yes."];
    assert_eq!(say(&sentences, 5), ["Hi.", "abcde", "fgh", "Yes."]);
}

#[test]
fn blank_text_is_nothing_to_say() {
    assert!(say(&["  \n "], MAX_UTTERANCE).is_empty());
    assert!(say(&[], MAX_UTTERANCE).is_empty());
    assert!(say(&["   ", "  "], 2).is_empty());
}
