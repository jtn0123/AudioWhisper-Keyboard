"""Break each safety, privacy and speed fix, require a real assertion failure, restore exact bytes, rerun green.

python3 -I scripts/hardening-mutations.py OUTPUT [CASE...]
Host only: cargo tests and Gradle JVM tests, no device. Compilation errors do not count. The
fixes are the settings presenter's buffer bounds, the auto-commit's cached lookup, read-aloud's
offline voice, the GPU renderer kept out of default builds (docs/decisions.md) and the keyboard
told its window is visible when shown again quickly (docs/rust.md). The runner is
glide-mutations.py's.
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
VOICES = CRATES + 'reader/src/voices.rs'
READER = 'lib/rust/src/main/kotlin/com/jtn0123/audiowhisper/rust/Reader.kt'
OFFLINE_VOICE = 'app/src/main/kotlin/com/jtn0123/audiowhisper/reader/OfflineVoice.kt'
IME_WINDOW = CRATES + 'awnative/src/ime_window.rs'
INPUT_VIEW = 'app/src/main/kotlin/com/jtn0123/audiowhisper/render/InputViewVisibility.kt'
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
    'read-aloud-network-voice-allowed': (
        VOICES,
        '!self.needs_network && self.installed', 'self.installed',
        ('cargo', 'reader', None, 'voices')),
    'read-aloud-uninstalled-voice-allowed': (
        VOICES,
        '!self.needs_network && self.installed', '!self.needs_network',
        ('cargo', 'reader', None, 'voices')),
    'read-aloud-any-default-kept': (
        VOICES,
        'voices.get(index).is_some_and(speaks)', 'index < voices.len()',
        ('cargo', 'reader', None, 'voices')),
    'read-aloud-other-language-reads': (
        VOICES,
        'facts.offline() && same(facts.language, language)', 'facts.offline()',
        ('cargo', 'reader', None, 'voices')),
    'read-aloud-facts-packed-swapped': (
        READER,
        '            numbers[at] = if (voice.needsNetwork) 1 else 0\n'
        '            numbers[at + 1] = if (voice.installed) 1 else 0\n',
        '            numbers[at] = if (voice.installed) 1 else 0\n'
        '            numbers[at + 1] = if (voice.needsNetwork) 1 else 0\n',
        ('gradle', ':app', '*OfflineVoiceTest')),
    'read-aloud-default-forgotten': (
        OFFLINE_VOICE,
        'language, country, voices.indexOf(default))', 'language, country, -1)',
        ('gradle', ':app', '*OfflineVoiceTest')),
    'quick-reshow-never-retold': (
        IME_WINDOW,
        '    window == VISIBLE && told != VISIBLE\n', '    false && window == VISIBLE && told != VISIBLE\n',
        ('cargo', 'awnative', None, 'ime_window')),
    'hidden-window-retold-visible': (
        IME_WINDOW,
        '    window == VISIBLE && told != VISIBLE\n', '    told != VISIBLE\n',
        ('cargo', 'awnative', None, 'ime_window')),
    'quick-reshow-retell-dropped': (
        INPUT_VIEW,
        '        if (retold) retell(View.VISIBLE)\n', '',
        ('gradle', ':app', '*InputViewVisibilityTest')),
    'gpu-renderer-on-by-default': (
        CRATES + 'render/Cargo.toml',
        '[features]\n', '[features]\ndefault = ["hybrid"]\n',
        ('cargo', 'render', None, 'the_gpu_renderer')),
}


if __name__ == '__main__':
    RUNNER.CASES = CASES
    RUNNER.main()
