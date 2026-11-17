"""Break each safety, privacy and speed fix, require a real assertion failure, restore exact bytes, rerun green.

python3 -I scripts/hardening-mutations.py OUTPUT [CASE...]
Host only: cargo tests and Gradle JVM tests, no device. Compilation errors do not count. The
fixes are the settings presenter's buffer bounds, the auto-commit's cached lookup and read-aloud's
offline voice (docs/decisions.md). The runner is glide-mutations.py's.
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
BOUNDS = CRATES + 'awnative/src/surface_bounds.rs'
SUGGESTER = 'app/src/main/kotlin/com/jtn0123/audiowhisper/suggest/LatinSuggester.kt'
# name: (file, before, after, how to test), as in glide-mutations.py.
CASES = {
    'settings-frame-height-unchecked': (
        BOUNDS,
        '        && frame_height == buffer.height\n', '',
        ('cargo', 'awnative', None, 'surface_bounds')),
    'settings-frame-width-unchecked': (
        BOUNDS,
        'let fits = frame_width == buffer.width\n        && ', 'let fits = ',
        ('cargo', 'awnative', None, 'surface_bounds')),
    'settings-stride-unchecked': (
        BOUNDS,
        '        && buffer.stride >= buffer.width\n', '',
        ('cargo', 'awnative', None, 'surface_bounds')),
    'settings-damage-unclamped': (
        BOUNDS,
        'region: damage.clamped(width, height),', 'region: damage,',
        ('cargo', 'awnative', None, 'surface_bounds')),
    'auto-commit-reuses-any-strip-for-the-word': (
        SUGGESTER,
        'last?.takeIf { it.inputs == inputs }', 'last?.takeIf { it.inputs.state.typed == inputs.state.typed }',
        ('gradle', ':app', '*AutoCorrectionReuseTest')),
    'auto-commit-ignores-learning': (
        SUGGESTER,
        'history?.changes() ?: 0)', '0L)',
        ('gradle', ':app', '*AutoCorrectionReuseTest')),
    'auto-commit-never-reuses': (
        SUGGESTER,
        '        last?.takeIf { it.inputs == inputs }?.let { return it.autoCorrection }\n', '',
        ('gradle', ':app', '*AutoCorrectionReuseTest')),
    'auto-commit-decodes-when-off': (
        SUGGESTER,
        '        if (inputs.strength == AutoCorrectStrength.OFF) return null\n', '',
        ('gradle', ':app', '*AutoCorrectionReuseTest')),
    'learned-words-uncounted': (
        CRATES + 'suggest/src/learned_words.rs',
        '    pub fn add(&mut self, word: &[u16], previous: Option<&[u16]>, keep_spelling: bool) {\n'
        '        self.changes = self.changes.wrapping_add(1);\n',
        '    pub fn add(&mut self, word: &[u16], previous: Option<&[u16]>, keep_spelling: bool) {\n',
        ('cargo', 'suggest', None, 'learned_words')),
}


if __name__ == '__main__':
    RUNNER.CASES = CASES
    RUNNER.main()
