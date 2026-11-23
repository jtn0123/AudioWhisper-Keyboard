"""Platform-edge faults (phase 5, docs/phase5-edges.md): each must trip a fresh assertion in at least
one of the test targets it names; source bytes are always restored. python3 -I.

    python3 -I scripts/edges-mutations.py build/edges/mutations [--part gif|dictation|reading] [case ...]

Run with no other Cargo or Gradle operation in this checkout, and do not read the source as the
truth while it runs: it is mutated then. A compilation failure never counts as caught: a Rust target
needs a "panicked at" line, a JVM target an AssertionFailedError in a fresh test report. Every target
a case names is run and recorded, so the result says which caught it.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent
GIF_RS = ROOT / 'lib/rust/crates/gif/src'
SESSION, RECENT = GIF_RS / 'session.rs', GIF_RS / 'session/recent.rs'
PREVIEWS, SEND, REPLY = GIF_RS / 'previews.rs', GIF_RS / 'send.rs', GIF_RS / 'reply.rs'
GIF_KT = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/gif'
SEARCH, IMAGES = GIF_KT / 'GifSearchSession.kt', GIF_KT / 'GifImages.kt'
SENDER, GIPHY = GIF_KT / 'GifSender.kt', GIF_KT / 'Giphy.kt'
# The targets: Rust test binaries, and JVM test classes run by one Gradle call.
GIF_CRATE, GIF_TRACE, GIF_UNITS = 'cargo:gif', 'jvm:gif-trace', 'jvm:gif-units'
GIF = [GIF_CRATE, GIF_TRACE]
GIF_CASES = [
    # The session: what each step shows, fetches, cancels, keeps and remembers.
    ('reset-keeps-previews', SESSION, '                effects.push(text(effect::FORGET_PREVIEWS));\n', '', GIF),
    ('refetch-while-loading', SESSION,
     'let showing = matches!(self.results, Results::Loaded(_)) || self.loading.is_some();',
     'let showing = matches!(self.results, Results::Loaded(_));', GIF),
    ('cancel-never-asked', SESSION, 'effects.extend([text(effect::CANCEL), text(loading.generation)]);',
     'let _ = loading;', GIF),
    ('cache-ignored', SESSION, 'if let Some(gifs) = self.cached(search, now) {',
     'if let Some(gifs) = self.cached(search, now).filter(|_| false) {', GIF),
    ('answer-not-kept', SESSION, '                self.keep(&loading.search, gifs.clone(), now);\n', '', GIF),
    ('trending-url-for-searches', SESSION, 'let url = if search.iter().copied().eq(TRENDING.encode_utf16()) {',
     'let url = if true {', GIF),
    ('opens-on-trending-not-recents', SESSION, 'let search = if self.trending() && !self.recents.is_empty() {',
     'let search = if false {', GIF),
    ('shown-without-a-source', SESSION, 'self.showing = show && self.available;', 'self.showing = show;', GIF),
    ('private-word-offered', SESSION, 'self.typed_word = if private || !has_text {',
     'self.typed_word = if !has_text {', GIF),
    ('incognito-remembered', SESSION, '        if incognito {\n            return Ok(());\n        }\n', '', GIF),
    ('editing-starts-from-trending', SESSION,
     'self.query = if self.trending() {\n                    Vec::new()\n',
     'self.query = if false {\n                    Vec::new()\n', GIF),
    ('swipe-from-the-current-query', SESSION, 'search_text::delete_last(&self.before_swipe, count)',
     'search_text::delete_last(&self.query, count)', GIF),
    ('submit-untrimmed', SESSION, 'let typed = chars::trim(&self.query).to_vec();',
     'let typed = self.query.clone();', GIF),
    ('recents-leave-the-fetch-running', SESSION,
     '        self.cancel(effects);\n        self.shown = crate::utf16(RECENTS);',
     '        self.shown = crate::utf16(RECENTS);', GIF),
    # Recents: read once per folder, merged with what was sent meanwhile, saved under their generation.
    ('reattach-rereads', RECENT, '        if self.path.as_deref() == Some(path) {\n            return;\n        }\n', '',
     GIF),
    ('merge-drops-the-sent', RECENT, 'let order = recents::load(&ids(&saved), &ids(&self.gifs), capacity);',
     'let order = recents::load(&ids(&saved), &[], capacity);', GIF),
    ('saved-under-the-wrong-generation', RECENT,
     'effects.extend([text(effect::SAVE), text(self.generation)]);',
     'effects.extend([text(effect::SAVE), text(self.generation.wrapping_add(1))]);', GIF),
    # Previews: 24 MB, least recently used first.
    ('previews-first-in-first-out', PREVIEWS,
     '        let entry = self.entries.remove(at);\n        self.entries.push(entry);\n', '        let _ = at;\n', GIF),
    ('previews-half-the-room', PREVIEWS, 'Previews::new(CAPACITY_BYTES)', 'Previews::new(CAPACITY_BYTES / 2)', GIF),
    # Sending: each step's next, the file's name, and what is stale.
    ('refused-gif-not-linked', SEND, '(START | COMMITTED, false) => link,',
     '(START, false) => link,\n        (COMMITTED, false) => &[FAILED],', GIF),
    ('cached-gif-not-touched', SEND, '(CHECKED, true) => &[TOUCH, COMMIT],', '(CHECKED, true) => &[COMMIT],', GIF),
    ('part-left-behind', SEND, '(SAVED | RENAMED, false) => &[DELETE_PART, FAILED],',
     '(SAVED | RENAMED, false) => &[FAILED],', GIF + [GIF_UNITS]),
    ('parts-kept-a-day', SEND, '        age > PART_STALE_MS\n', '        age > KEEP_MS\n', GIF),
    ('file-name-keeps-symbols', SEND, '.filter(|&unit| chars::is_letter_or_digit(unit))', '.filter(|_| true)', GIF),
    # Replies: how each kind of answer reads, and each request's cap.
    ('too-large-is-unreachable', REPLY, 'reply::TOO_LARGE => Err(Failure::Unreadable),',
     'reply::TOO_LARGE => Err(Failure::Unreachable),', GIF),
    ('previews-capped-as-sends', REPLY, 'media::PREVIEW => MAX_PREVIEW_BYTES,', 'media::PREVIEW => MAX_SEND_BYTES,',
     GIF),
    # The Kotlin that runs each effect.
    ('fetch-not-cancelled', SEARCH, 'fetches.remove(answer[at + 1].toInt())?.cancel()',
     'fetches.remove(answer[at + 1].toInt())', [GIF_TRACE]),
    ('too-large-reported-unreachable', SEARCH, 'Triple(GifReply.TOO_LARGE, 0, "")',
     'Triple(GifReply.UNREACHABLE, 0, "")', [GIF_TRACE, GIF_UNITS]),
    ('merge-not-shown', SEARCH, '            run(GifOp.MERGED)\n', '', [GIF_TRACE, GIF_UNITS]),
    ('recents-saved-on-main', SEARCH, '        scope.launch(io) {\n            val rows = plan.rows(generation)',
     '        scope.launch {\n            val rows = plan.rows(generation)', [GIF_TRACE]),
    ('preview-never-kept', IMAGES, 'held(url) ?: download(url).also { keep(url, it) }', 'held(url) ?: download(url)',
     [GIF_TRACE]),
    ('touch-skipped', SENDER, 'GifSendAction.TOUCH -> withContext(io) { files.file.setLastModified(files.now) }.let { null }',
     'GifSendAction.TOUCH -> null', [GIF_TRACE, GIF_UNITS]),
    ('reply-uncapped', GIPHY, 'readLimited(it, request[GifRequest.MAX_BYTES])', 'readLimited(it, Int.MAX_VALUE)',
     [GIF_UNITS]),
    ('refusal-reported-as-ok', GIPHY, 'if (status != request[GifRequest.OK_STATUS]) return read(status, null)',
     'if (status != request[GifRequest.OK_STATUS]) return read(request[GifRequest.OK_STATUS], null)', [GIF_UNITS]),
]
DICT_RS = ROOT / 'lib/rust/crates/dictation/src'
AUDIO, MODEL = DICT_RS / 'audio.rs', DICT_RS / 'model_check.rs'
DICT_KT = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/dictation'
MIC_KT, MODEL_KT = DICT_KT / 'MicRecorder.kt', DICT_KT / 'ModelCheck.kt'
DICT_CRATE, MIC_TRACE, MODEL_TRACE = 'cargo:dictation', 'jvm:mic-trace', 'jvm:model-trace'
MIC, MODELS = [DICT_CRATE, MIC_TRACE], [DICT_CRATE, MODEL_TRACE]
DICT_UNITS = 'jvm:dictation-units'
DICTATION_CASES = [
    # The microphone: how loud a chunk is, what a read means, the buffer, and a WAV file's samples.
    ('level-factor-seven', AUDIO, '.sqrt() * 8.0;', '.sqrt() * 7.0;', MIC),
    ('level-uncapped', AUDIO, '(rms as f32).clamp(0.0, 1.0)', 'rms as f32', MIC),
    ('error-ends-quietly', AUDIO, '..0 if running => read::REPORT,', '..0 if running => read::END,', MIC),
    ('error-after-stop-reported', AUDIO, '..0 => read::END,', '..0 => read::REPORT,', MIC),
    ('nothing-delivered', AUDIO, '0 => read::SKIP,', '0 => read::DELIVER,', MIC),
    ('buffer-quarter-second', AUDIO, 'size_of::<f32>() / 2) as i32', 'size_of::<f32>() / 4) as i32', MIC),
    ('scale-32767', AUDIO, 'i16::from_le_bytes(pair)) / 32768.0', 'i16::from_le_bytes(pair)) / 32767.0', MIC),
    ('twelve-bytes-a-wav', AUDIO, 'wav.len() <= 12 ||', 'wav.len() < 12 ||', MIC),
    ('odd-chunks-unpadded', AUDIO, '8i32.wrapping_add(size).wrapping_add(size & 1)', '8i32.wrapping_add(size)', MIC),
    ('bytes-read-as-samples', AUDIO, 'size.min(available) / 2)', 'size.min(available) / 4)', MIC),
    ('walk-stops-early', AUDIO, 'for _ in 0..=wav.len() {', 'for _ in 0..2 {', MIC),
    ('kt-error-ends-quietly', MIC_KT, 'MicRead.REPORT -> return onError(n)', 'MicRead.REPORT -> return',
     [MIC_TRACE, DICT_UNITS]),
    ('kt-level-of-whole-buffer', MIC_KT, 'MicRules.level(buffer, n)', 'MicRules.level(buffer, buffer.size)',
     [MIC_TRACE, DICT_UNITS]),
    # A model's files: which are missing or damaged, which are hashed, and which are remembered.
    ('damaged-wins', MODEL, 'if !missing.is_empty() {', 'if false {', MODELS),
    ('key-without-time', MODEL, 'format!(":{length}:{modified}")', 'format!(":{length}")', MODELS),
    ('known-files-hashed', MODEL, '.filter(|&i| !known[i]).collect()', '.filter(|&i| known[i]).collect()', MODELS),
    ('known-files-judged', MODEL, '        if known[i] {\n            continue;\n        }\n', '', MODELS),
    ('sums-unchecked', MODEL, 'Some(sum) if *sum == expected[i] => passed.push(i),', 'Some(_) => passed.push(i),',
     MODELS),
    ('failures-remembered', MODEL, '_ => damaged.push(i),', '_ => {\n                damaged.push(i);\n'
     '                passed.push(i);\n            }', MODELS),
    ('kt-state-row-remembered', MODEL_KT, 'for (at in 2 + verdict[1] until verdict.size)',
     'for (at in 2 until verdict.size)', [MODEL_TRACE, DICT_UNITS]),
    ('kt-names-shifted', MODEL_KT, 'val names = (2 until 2 + answer[1])', 'val names = (1 until 1 + answer[1])',
     [MODEL_TRACE, DICT_UNITS]),
]
READER_RS = ROOT / 'lib/rust/crates/reader/src'
PLAN, LOOK, WINDOW = READER_RS / 'reading.rs', READER_RS / 'reading/look.rs', READER_RS / 'window.rs'
READING_NATIVE = ROOT / 'lib/rust/crates/awnative/src/reading.rs'
READING_KT = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/reader/Reading.kt'
READER_CRATE, READING_TRACE = 'cargo:reader', 'jvm:reading-trace'
READ = [READER_CRATE, READING_TRACE]
READING_CASES = [
    # Looking: the walks, their retries, and a web page settling.
    ('look-tries-seven', PLAN, 'pub const LOOK_TRIES: u32 = 8;', 'pub const LOOK_TRIES: u32 = 7;', READ),
    ('retry-wait-shorter', PLAN, 'pub const LOOK_RETRY_MS: i32 = 250;', 'pub const LOOK_RETRY_MS: i32 = 200;', READ),
    ('settle-tries-five', PLAN, 'pub const SETTLE_TRIES: u32 = 4;', 'pub const SETTLE_TRIES: u32 = 5;', READ),
    ('empty-screen-kept', LOOK, 'screen.filter(|it| !it.texts.is_empty())', 'screen.filter(|_| true)', READ),
    ('every-page-settled', LOOK, 'let settle = screen.web && screen.all_shown();',
     'let settle = screen.all_shown();', READ),
    ('settled-on-equal-nodes', LOOK, 'screen.filter(|it| it.nodes > nodes)', 'screen.filter(|it| it.nodes >= nodes)',
     READ),
    ('selection-reads-own-window', PLAN, 'At::Begun if yes => self.look(self.from == from::SELECTION, Then::Start),',
     'At::Begun if yes => self.look(false, Then::Start),', READ),
    # The voice: a failure said, one that can't speak told long, one that won't begin.
    ('failed-voice-unsaid', PLAN, 'At::Took if value & 2 != 0 => {', 'At::Took if value & 4 != 0 => {', READ),
    ('unavailable-told-short', PLAN, 'effect::TELL, message::UNAVAILABLE, 1)', 'effect::TELL, message::UNAVAILABLE, 0)',
     READ),
    ('unbegun-read-anyway', PLAN, 'At::Begun if yes =>', 'At::Begun if yes || value == 0 =>', READ),
    # Where reading starts: in sight, at the top, at the selection.
    ('screen-from-first-block', PLAN, 'block: first.unwrap_or(0),', 'block: first.map_or(0, |_| 0),', READ),
    ('top-not-revealed', PLAN, 'Some(block) => self.emit(At::Revealed, effect::REVEAL, block as i32, 0),',
     'Some(_) => self.after_reveal(),', READ),
    ('web-scrolled-back', PLAN, 'if !self.screen.web && scrolls < MAX_SCROLLS_BACK {',
     'if scrolls < MAX_SCROLLS_BACK {', READ),
    ('scroll-back-cap', PLAN, 'pub const MAX_SCROLLS_BACK: u32 = 60;', 'pub const MAX_SCROLLS_BACK: u32 = 61;', READ),
    # Reading on: two blocks ahead, the offset once, what is remembered, the scroll on.
    ('lookahead-three', PLAN, 'LOOKAHEAD as i32 - 1,', 'LOOKAHEAD as i32,', READ),
    ('offset-kept', PLAN, 'let offset = std::mem::take(&mut self.offset);', 'let offset = self.offset;', READ),
    ('page-remembered', PLAN, 'if self.screen.scrolls[block] {', 'if block < usize::MAX {', READ),
    ('recent-uncapped', PLAN, 'if self.read.len() > RECENT {', 'if self.read.len() > RECENT * 100 {', READ),
    ('other-app-carried-on', PLAN, 'if self.screen.app != self.app {', 'if self.app.is_some() && self.app.is_none() {',
     READ),
    ('scroll-tries-four', PLAN, '} else if tries < TRIES_AFTER_SCROLL {', '} else if tries <= TRIES_AFTER_SCROLL {',
     READ),
    ('scroll-settle-shorter', PLAN, 'pub const SETTLE_MS: i32 = 350;', 'pub const SETTLE_MS: i32 = 300;', READ),
    # The window a reading reads.
    ('window-last-top-layer', WINDOW, 'top.filter(|&t| layers[t] >= layers[i])', 'top.filter(|&t| layers[t] > layers[i])',
     READ),
    ('window-focused-first', WINDOW, 'let app = (apps().find(|&i| active[i]))', 'let app = (apps().find(|&i| focused[i]))',
     READ),
    ('keyboard-above-cuts', WINDOW, 'if keyboard_top > top && keyboard_top < bottom {', 'if keyboard_top < bottom {',
     READ),
    # The natives and the Kotlin runner.
    ('native-visible-swapped', READING_NATIVE, 'fact::VISIBLE_TOP,\n        fact::VISIBLE_BOTTOM,',
     'fact::VISIBLE_BOTTOM,\n        fact::VISIBLE_TOP,', [READING_TRACE]),
    ('native-web-dropped', READING_NATIVE, 'web: web != 0,', 'web: web > 1,', [READING_TRACE]),
    ('kt-voice-bits-swapped', READING_KT, 'bit(host.takeVoice()) or 2 * bit(host.voiceFailedJustNow())',
     '2 * bit(host.takeVoice()) or bit(host.voiceFailedJustNow())', [READING_TRACE]),
    ('kt-skip-own-flipped', READING_KT, 'look(plan, skipOwn = effect[1] == 1)', 'look(plan, skipOwn = effect[1] == 0)',
     [READING_TRACE]),
    ('kt-scroll-backwards', READING_KT, 'scroll(forward = effect[0] == ReadingEffect.SCROLL_FORWARD)',
     'scroll(forward = effect[0] == ReadingEffect.SCROLL_BACK)', [READING_TRACE]),
    ('kt-keep-ignored', READING_KT, 'ReadingEffect.KEEP -> screen = walked', 'ReadingEffect.KEEP -> Unit',
     [READING_TRACE]),
    ('kt-whole-block-queued', READING_KT, 'voice().queue(block.text.substring(effect[2]))', 'voice().queue(block.text)',
     [READING_TRACE]),
    ('kt-web-fact-dropped', READING_KT, 'facts[ReadingFact.WEB] = bit(found.web)', 'facts[ReadingFact.WEB] = 0',
     [READING_TRACE]),
    ('kt-end-keeps-voice', READING_KT, '                    host.idleVoice()\n', '', [READING_TRACE]),
]
PARTS = {'gif': GIF_CASES, 'dictation': DICTATION_CASES, 'reading': READING_CASES}
CARGO = {GIF_CRATE: ['-p', 'gif'], DICT_CRATE: ['-p', 'dictation'], READER_CRATE: ['-p', 'reader']}
JVM = {
    READING_TRACE: ['*ReadingTraceTest'],
    MIC_TRACE: ['*MicTraceTest'],
    MODEL_TRACE: ['*ModelCheckTraceTest'],
    DICT_UNITS: ['*ModelCheckTest', '*MicRecorderTest', '*AudioHistoryTest'],
    GIF_TRACE: ['*GifSearchTraceTest'],
    GIF_UNITS: ['*GifRecentsTest', '*GifChipsTest', '*GiphyTest', '*GifSenderFilesTest', '*GiphyTransportTest'],
}
REPORTS = ROOT / 'app/build/test-results/testDebugUnitTest'


def env():
    e = dict(os.environ)
    e.setdefault('CARGO_BUILD_JOBS', '6')
    if 'ANDROID_HOME' not in e:
        for line in (ROOT / 'local.properties').read_text().splitlines():
            if line.startswith('sdk.dir='):
                e['ANDROID_HOME'] = line.split('=', 1)[1]
    return e


def rust(output, name, target):
    command = ['cargo', 'test', '-q'] + CARGO[target]
    log_path = output / (name + '.' + target.replace(':', '-') + '.log')
    with log_path.open('wb') as log:
        result = subprocess.run(command, cwd=ROOT / 'lib/rust', stdout=log, stderr=subprocess.STDOUT, env=env())
    text = log_path.read_text(errors='replace')
    return result.returncode, sum('panicked at' in line for line in text.splitlines())


def jvm(output, name, target):
    started = time.time()
    command = ['./gradlew', '--stacktrace', ':app:testDebugUnitTest']
    for pattern in JVM[target]:
        command += ['--tests', pattern]
    with (output / (name + '.' + target.replace(':', '-') + '.log')).open('wb') as log:
        result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=env())
    failures = 0
    for report in REPORTS.glob('TEST-*.xml'):
        if report.stat().st_mtime >= started:
            failures += sum('AssertionFailedError' in f.attrib.get('type', '')
                            for f in ET.parse(report).getroot().iter('failure'))
    return result.returncode, failures


def run(output, name, target):
    return (rust if target.startswith('cargo:') else jvm)(output, name, target)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--part', choices=sorted(PARTS), action='append', help='parts to run (default: all)')
    parser.add_argument('cases', nargs='*', help='case names to run (default: all in the parts)')
    args = parser.parse_args()
    every = [case for part in (args.part or sorted(PARTS)) for case in PARTS[part]]
    unknown = set(args.cases) - {case[0] for case in every}
    assert not unknown, f'unknown cases: {sorted(unknown)}'
    cases = [case for case in every if not args.cases or case[0] in args.cases]
    output = ROOT / args.output
    output.mkdir(parents=True, exist_ok=False)
    originals = {path: path.read_bytes() for _, path, _, _, _ in cases}
    for name, path, old, _, _ in cases:
        assert originals[path].decode().count(old) == 1, 'mutation location changed: ' + name
    results = []
    try:
        for name, path, old, new, targets in cases:
            path.write_text(originals[path].decode().replace(old, new))
            try:
                outcome = {}
                for target in targets:
                    status, failures = run(output, name, target)
                    outcome[target] = {'exit': status, 'assertion_failures': failures}
            finally:
                path.write_bytes(originals[path])
            caught = [t for t, o in outcome.items() if o['exit'] != 0 and o['assertion_failures'] > 0]
            results.append({'mutation': name, 'file': str(path.relative_to(ROOT)), 'targets': outcome,
                            'caught_by': caught})
            (output / 'result.json').write_text(json.dumps({'mutations': results}, indent=2) + '\n')
            print(name + ': ' + ('caught by ' + ', '.join(caught) if caught else 'NOT CAUGHT'), flush=True)
    finally:
        for path, original in originals.items():
            path.write_bytes(original)
    restored = {}
    for target in sorted({t for case in cases for t in case[4]}):
        status, failures = run(output, 'restored', target)
        restored[target] = {'exit': status, 'assertion_failures': failures}
    missed = [r['mutation'] for r in results if not r['caught_by']]
    report = {'mutations': results, 'missed': missed, 'restored': restored,
              'source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in originals}}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    assert all(r['exit'] == 0 and r['assertion_failures'] == 0 for r in restored.values()), 'restored source fails'
    assert not missed, f'not caught: {missed}'
    print(f'All {len(results)} mutations caught; restored source passes {sorted(restored)}', flush=True)


if __name__ == '__main__':
    main()
