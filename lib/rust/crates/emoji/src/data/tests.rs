use super::*;
use crate::utf16;

const IDS: [&str; 3] = ["recently_used", "smileys", "people"];

fn parse(text: &str) -> EmojiData {
    let ids: Vec<Vec<u16>> = IDS.iter().map(|id| utf16(id)).collect();
    let ids: Vec<&[u16]> = ids.iter().map(Vec::as_slice).collect();
    EmojiData::parse(utf16(text), &ids)
}

fn string(data: &EmojiData, span: Span) -> String {
    String::from_utf16_lossy(data.units(span))
}

/// Each entry as `value/name/keyword,keyword`.
fn entries(data: &EmojiData) -> Vec<String> {
    data.entries
        .iter()
        .map(|e| {
            let keywords: Vec<String> = e.keywords.iter().map(|&k| string(data, k)).collect();
            format!(
                "{}/{}/{}",
                string(data, e.value),
                string(data, e.name),
                keywords.join(",")
            )
        })
        .collect()
}

fn sets(data: &EmojiData) -> Vec<(usize, usize, usize)> {
    data.sets
        .iter()
        .map(|s| (s.category, s.start, s.len))
        .collect()
}

#[test]
fn lines_end_as_a_buffered_reader_ends_them() {
    let text = utf16("a\nb\r\nc\rd\r\r\ne\n");
    let cut: Vec<String> = lines(&text)
        .map(|s| String::from_utf16_lossy(s.of(&text)))
        .collect();
    assert_eq!(cut, ["a", "b", "c", "d", "", "e"]);
    assert_eq!(lines(&[]).count(), 0);
    assert_eq!(lines(&utf16("\n")).count(), 1);
    assert_eq!(lines(&utf16("x")).count(), 1);
}

#[test]
fn a_file_parses_into_sets_and_skin_tones() {
    let data = parse(
        "# comment\n[smileys]\n😀;grinning face;face|grin\n\n👋;waving hand;hand|wave\n\t👋🏻; light ;x\n\t👋🏿;dark;y\n[people]\n🧑;person; a | b |\n",
    );
    assert_eq!(
        entries(&data),
        [
            "😀/grinning face/face,grin",
            "👋/waving hand/hand,wave",
            "👋🏻/waving hand/x",
            "👋🏿/waving hand/y",
            "🧑/person/a,b,",
        ]
    );
    assert_eq!(sets(&data), [(1, 0, 1), (1, 1, 3), (2, 4, 1)]);
    assert_eq!(data.by_skin_tone[0], [0, 1, 4]);
    assert_eq!(data.by_skin_tone[1], [0, 2, 4]);
    assert_eq!(data.by_skin_tone[3], [0, 4]);
    assert_eq!(data.by_skin_tone[5], [0, 3, 4]);
    assert_eq!(data.entries[2].skin_tone, 1);
}

#[test]
fn categories_go_in_the_ids_order_and_may_repeat() {
    let data = parse("[people]\nA;a;\n[smileys]\nB;b;\n[people]\nC;c;\n");
    assert_eq!(sets(&data), [(2, 0, 1), (1, 1, 1), (2, 2, 1)]);
    assert_eq!(data.by_skin_tone[2], [1, 0, 2]);
}

#[test]
fn odd_lines_are_skipped_as_kotlin_skipped_them() {
    let data = parse(
        "A;before any category;\n[unknown]\nB;b;\n[\n[smileys\nC;c;\n[smileys]\n#️⃣;keycap: #;keycap\n D ; d ;k\n\tE;e;k\n  \n\tF;f;k\nG;g\n\tH;h;k;\n\tI;i;k\nJ;j;k\r\n",
    );
    // "[smileys" is "smiley", unknown; the keycap is a comment; the blank line keeps the set open;
    // "G;g" ends the set and is dropped, so "I" starts one of its own.
    assert_eq!(
        entries(&data),
        ["D/d/k", "E/d/k", "F/d/k", "I/i/k", "J/j/k"]
    );
    assert_eq!(sets(&data), [(1, 0, 3), (1, 3, 1), (1, 4, 1)]);
    // The first set has three emoji, all without a tone: none is shown for another tone.
    assert_eq!(data.by_skin_tone[0], [0, 1, 2, 3, 4]);
    assert_eq!(data.by_skin_tone[4], [3, 4]);
}

#[test]
fn a_set_keeps_its_category_when_the_next_bracket_ends_it() {
    let data = parse("[smileys]\nA;a;\n\tB;b;\n[recently_used]\n\tC;c;\n");
    assert_eq!(sets(&data), [(1, 0, 2), (0, 2, 1)]);
    // C, alone, is shown for every tone, and recently_used comes first.
    assert_eq!(data.by_skin_tone[0], [2, 0, 1]);
}

#[test]
fn empty_text_has_nothing() {
    let data = parse("");
    assert!(data.entries.is_empty() && data.sets.is_empty());
    assert!(data.by_skin_tone.iter().all(Vec::is_empty));
}
