"""Reading aloud's rules, screen walk and play queue (ReaderRulesMatchTest, TtsChunkMatchTest,
ScreenRulesMatchTest, ScreenTraversalMatchTest, PlaybackQueueMatchTest). The reading's steps, the
window it reads and the voice's hold are covered by edges-mutations.py --part reading and voice;
chunking at the limit, stop-before-free and the password rule by tts-mutations.py tts."""
READER = 'lib/rust/crates/reader/src/'
KOTLIN = 'app/src/main/kotlin/com/jtn0123/audiowhisper/reader/'
MODULE = ':app'
TARGETS = {
    'rules': ['*ReaderRulesMatchTest', '*TtsChunkMatchTest'],
    'screen': ['*ScreenRulesMatchTest'],
    'walk': ['*ScreenTraversalMatchTest'],
    'queue': ['*PlaybackQueueMatchTest'],
}
CASES = [
    # Pages: what shows, where reading starts, where a selection is.
    ('shown-when-touching', READER + 'page_text.rs', 'self.bottom > self.top && self.bottom > visible.top',
     'self.bottom > self.top && self.bottom >= visible.top', 'rules'),
    ('last-off-screen-match', READER + 'page_text.rs', 'first = first.or(Some(start));', 'first = Some(start);', 'rules'),
    ('space-runs-kept', READER + 'page_text.rs', '        if space && last_space {\n',
     '        if space && last_space && unit == SPACE {\n', 'rules'),
    # Pieces joined into blocks: what joins them, the bullets dropped, the edges kept.
    ('brace-joins-nothing', READER + 'page_text.rs', "    b'}' as u16,\n", "    b'{' as u16,\n", 'rules'),
    ('middle-dot-kept', READER + 'page_text.rs', '0x2023, 0x2043, 0x00B7, SPACE', '0x2023, 0x2043, SPACE', 'rules'),
    ('flat-pieces-count', READER + 'page_text.rs', '.filter(|it| it.bottom > it.top)', '.filter(|it| it.bottom >= it.top)',
     'rules'),
    # After a scroll: the anchor, and the recent blocks skipped.
    ('shorter-anchor', READER + 'page_text.rs', 'const ANCHOR: usize = 3;', 'const ANCHOR: usize = 2;', 'rules'),
    ('one-fewer-recent', READER + 'page_text.rs', 'read[read.len().saturating_sub(RECENT)..]',
     'read[read.len().saturating_sub(RECENT - 1)..]', 'rules'),
    # Text to speak: joined while it fits, cut at a space, the app's limit.
    ('joined-short-of-max', READER + 'page_text.rs', 'current.len() + sentence.len() > max',
     'current.len() + sentence.len() >= max', 'rules'),
    ('cut-at-a-leading-space', READER + 'page_text.rs', '.filter(|&it| it > 0)', '.filter(|&it| it >= 0)', 'rules'),
    ('limit-moved', READER + 'page_text.rs', 'pub const MAX_UTTERANCE: usize = 3_900;',
     'pub const MAX_UTTERANCE: usize = 3_901;', 'rules'),
    # Keeping the words in view, and the speaker key.
    ('fits-from-the-middle', READER + 'follow_along.rs', 'first < top.wrapping_add(bottom) / 2',
     'first <= top.wrapping_add(bottom) / 2', 'rules'),
    ('fits-only-below-the-top', READER + 'follow_along.rs', 'first >= top && (', 'first > top && (', 'rules'),
    ('hold-exclusive', READER + 'speaker_key.rs', 'now.wrapping_sub(at) >= HOLD_MS', 'now.wrapping_sub(at) > HOLD_MS',
     'rules'),
    ('hold-saturating', READER + 'speaker_key.rs', 'now.wrapping_sub(at) >= HOLD_MS', 'now.saturating_sub(at) >= HOLD_MS',
     'rules'),
    ('stop-keeps-the-press', READER + 'speaker_key.rs', '            self.down_at = None;\n            return Some',
     '            return Some', 'rules'),
    # The screen's rules: boxes, sideways views, the list, moves.
    ('touching-boxes-overlap', READER + 'screen.rs', 'let overlaps = self.left < other.right',
     'let overlaps = self.left <= other.right', 'screen'),
    ('area-saturates', READER + 'screen.rs', 'width.wrapping_mul(', 'width.saturating_mul(', 'screen'),
    ('one-column-sideways', READER + 'screen.rs', 'rows == 1 && columns > 1', 'rows == 1 && columns >= 1', 'screen'),
    ('pager-only-at-the-end', READER + 'screen.rs', '|| class_name.windows(pager.len()).any(|it| it == pager))',
     '|| class_name.ends_with(&pager))', 'screen'),
    ('tie-not-the-bigger', READER + 'screen.rs', '(holds[index], area) > (holds[it], areas[it])',
     'holds[index] > holds[it]', 'screen'),
    ('unseen-scroller-is-the-list', READER + 'screen.rs', '.filter(|&(_, &area)| area > 0)',
     '.filter(|&(_, &area)| area >= 0)', 'screen'),
    ('above-turns-pages', READER + 'screen.rs', '} else if first < top || !can_turn {', '} else if !can_turn {', 'screen'),
    ('after-turn-at-the-top', READER + 'screen.rs', '    if first < top {\n        AfterTurn::ShowOnScreen',
     '    if first <= top {\n        AfterTurn::ShowOnScreen', 'screen'),
    ('inside-at-the-edge', READER + 'screen.rs', 'top > view_top && bottom < view_bottom',
     'top >= view_top && bottom < view_bottom', 'screen'),
    # The walk of the screen: its limits, its pieces.
    ('one-level-shallower', READER + 'walk.rs', 'depth > MAX_DEPTH', 'depth >= MAX_DEPTH', 'walk'),
    ('one-node-more', READER + 'walk.rs', 'self.nodes >= MAX_NODES', 'self.nodes > MAX_NODES', 'walk'),
    ('blank-text-read', READER + 'walk.rs', 'text.iter().any(|c| !is_whitespace(*c))', '!text.is_empty()', 'walk'),
    ('piece-in-its-own-scroller', READER + 'walk.rs', '                    bounds: node.bounds,\n                    scroller,\n',
     '                    bounds: node.bounds,\n                    scroller: inner,\n', 'walk'),
    # The play queue: when a block is heard, when it is done, when it began.
    ('heard-only-past-the-start', READER + 'play_queue.rs', 'if frame <= start && !over {', 'if frame < start && !over {',
     'queue'),
    ('done-only-past-the-end', READER + 'play_queue.rs', '.is_some_and(|b| b.end.is_some_and(|end| frame >= end))',
     '.is_some_and(|b| b.end.is_some_and(|end| frame > end))', 'queue'),
    ('start-moves', READER + 'play_queue.rs', 'b.start.get_or_insert(frame);', 'b.start = Some(frame);', 'queue'),
    ('kt-first-start-dropped', KOTLIN + 'PlayQueue.kt', 'result.drop(PlaybackHeard.SIZE)', 'result.drop(PlaybackHeard.SIZE + 1)',
     'queue'),
]
