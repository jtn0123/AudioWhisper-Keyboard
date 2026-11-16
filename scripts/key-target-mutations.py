"""Break each key-targeting rule, require a real assertion failure, restore exact bytes, rerun green.

python3 -I scripts/key-target-mutations.py OUTPUT [CASE...]
Host only: cargo tests and Gradle JVM tests, no device. Compilation errors do not count. The
rules are invisible key resizing by the letter odds (docs/key-resize-results.md) and keys pressed
together typing in the order they went down. The runner is glide-mutations.py's.
"""
import importlib.util
import sys
from pathlib import Path

# Loading the runner as a module would otherwise leave a __pycache__ in scripts/.
sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location('glide_mutations', Path(__file__).parent / 'glide-mutations.py')
RUNNER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUNNER)

CRATES = RUNNER.CRATES
RESIZE = CRATES + 'keys/src/resize.rs'
ROLL = CRATES + 'render/src/keyboard_touch.rs'
# name: (file, before, after, how to test), as in glide-mutations.py.
CASES = {
    'border-move-uncapped': (
        RESIZE,
        'let moved = (factor * (theirs - own)).min(tuning.cap);',
        'let moved = factor * (theirs - own);',
        ('cargo', 'keys', None, 'resize')),
    'border-moves-towards-the-less-likely': (
        RESIZE,
        'let moved = (factor * (theirs - own)).min(tuning.cap);',
        'let moved = (factor * (own - theirs)).min(tuning.cap);',
        ('cargo', 'keys', None, 'resize')),
    'odds-without-the-aim': (
        RESIZE,
        'let aimed = crate::aim::retarget(hit, (x, y), &shift, &key_at);',
        'let aimed = hit;',
        ('cargo', 'keys', None, 'resize')),
    'odds-from-the-raw-tap': (
        RESIZE,
        '(x - dx).clamp(r.left, r.right),', 'x.clamp(r.left, r.right),',
        ('cargo', 'keys', None, 'resize')),
    'odds-past-the-border': (
        RESIZE,
        '(x - dx).clamp(r.left, r.right),', 'x - dx,',
        ('cargo', 'keys', None, 'resize')),
    'next-units-without-backoff': (
        CRATES + 'ngram/src/letters.rs',
        'explicit3 + levels.trigram_backoff * (explicit2 + levels.bigram_backoff * rest)',
        'explicit3 + explicit2 + rest',
        ('cargo', 'ngram', None, 'next_units')),
    'priors-for-every-key': (
        CRATES + 'suggest/src/letter_odds.rs',
        'letter.then(|| self.prior(code, Self::FLOOR))', 'Some(self.prior(code, Self::FLOOR))',
        ('cargo', 'suggest', None, 'letter_odds')),
    'odds-never-stale': (
        CRATES + 'suggest/src/letter_odds.rs',
        'trail.at_word(&self.word).then_some(&self.odds)', 'Some(&self.odds)',
        ('cargo', 'suggest', None, 'letter_odds')),
    'odds-hold-under-a-pressed-key': (
        CRATES + 'suggest/src/tap.rs',
        'self.pressed_code_point == NONE && self.typed_for(word).is_some()',
        'self.typed_for(word).is_some()',
        ('cargo', 'suggest', None, 'what_was_worked_out')),
    'first-letter-odds-without-a-space': (
        CRATES + 'suggest/src/tap.rs',
        'return self.pressed_code_point == SPACE;', 'return true;',
        ('cargo', 'suggest', None, 'what_was_worked_out')),
    'renderer-ignores-odds': (
        CRATES + 'render/src/scene.rs',
        'let prior = |index: usize| self.priors.get(index).copied().flatten();',
        'let prior = |_: usize| None;',
        ('cargo', 'render', 'aimed_keys', '')),
    'suggester-keeps-no-odds': (
        CRATES + 'awnative/src/suggest/session.rs',
        'super::letter_odds::note(&session, &state);', 'let _ = (&session, &state);',
        ('gradle', ':lib:rust', '*KeyResizeTest')),
    'compose-hit-test-ignores-odds': (
        CRATES + 'awnative/src/suggest/tap_aim.rs',
        'let priors = super::letter_odds::priors(&taps.trail, codes.iter().copied());',
        'let priors: Vec<Option<f32>> = Vec::new();',
        ('gradle', ':lib:rust', '*KeyResizeTest')),
    'renderer-touch-ignores-odds': (
        CRATES + 'awnative/src/suggest/tap_aim.rs',
        'scene.priors = super::letter_odds::priors(&taps.trail, scene.keys.iter().map(|key| key.code));',
        'scene.priors.clear();',
        ('gradle', ':lib:rust', '*KeyResizeTest')),
    'held-space-never-rolls': (
        CRATES + 'input/src/pointer.rs',
        'matches!(code, SPACE | CJK_SPACE | DELETE)', 'matches!(code, CJK_SPACE | DELETE)',
        ('cargo', 'render', 'fast_typing', '')),
    'held-delete-never-rolls': (
        CRATES + 'input/src/pointer.rs',
        'matches!(code, SPACE | CJK_SPACE | DELETE)', 'matches!(code, SPACE | CJK_SPACE)',
        ('cargo', 'render', 'fast_typing', '')),
    'swiping-space-rolls': (
        ROLL, 'let still = !p.moved && !p.long;', 'let still = !p.long;',
        ('cargo', 'render', 'fast_typing', '')),
    'long-held-space-rolls': (
        ROLL, 'let still = !p.moved && !p.long;', 'let still = !p.moved;',
        ('cargo', 'render', 'fast_typing', '')),
    'compose-space-never-rolls': (
        CRATES + 'awnative/src/input.rs',
        '[code, still] => input::pointer::rolls(*code as i32, *still != 0.0),',
        '[code, _] => input::pointer::rolls(*code as i32, false),',
        ('gradle', ':app', '*InputPolicyMatchTest')),
}


if __name__ == '__main__':
    RUNNER.CASES = CASES
    RUNNER.main()
