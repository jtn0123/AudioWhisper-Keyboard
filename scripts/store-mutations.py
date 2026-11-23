"""Store faults (all-rust phase 5, docs/phase5-stores.md): each must trip a fresh assertion in at least one of
the test targets it names; source bytes are always restored. python3 -I.

    python3 -I scripts/store-mutations.py build/stores/mutations [case ...]
    python3 -I scripts/store-mutations.py --check                 # only that every location is still there
    python3 -I scripts/store-mutations.py --targets cargo:stores,jvm:stores build/stores/fast

The targets: `cargo test -p stores`; the JVM match tests against main's Room (ClipboardStoreMatchTest,
UserWordStoreMatchTest); and the device tests on emulator-5556 (StoresMigrationDeviceTest,
StoresBackupDeviceTest), which build the app with -PstoresFixture=true, install it with scripts/install-dev.sh and
run the instrumentation. `--targets` runs only those named, for a quick pass; a case none of whose targets ran
is then skipped, not counted as caught.

Run with no other Cargo or Gradle operation in this checkout, and nothing else on emulator-5556: the source is
mutated while it runs. A compilation failure never counts as caught: a Rust target needs a "panicked at" line, a
JVM target an assertion failure in a fresh test report, a device target an assertion failure in the
instrumentation's output. Logs stay in the output folder and are never printed: the fixtures' text is synthetic,
but nothing a test read is echoed.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import xml.etree.ElementTree as ET

sys.dont_write_bytecode = True  # importing mutation_cases would otherwise leave a __pycache__ in scripts/
sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutation_cases

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / 'lib/rust/crates/stores/src'
CODEC, CLIP_CODEC, WORD_CODEC = SRC / 'codec.rs', SRC / 'clip_codec.rs', SRC / 'word_codec.rs'
DISK, STORE, LEGACY, CLIPS, WORDS = (SRC / name for name in ('disk.rs', 'store.rs', 'legacy.rs', 'clips.rs', 'words.rs'))
BRIDGE = ROOT / 'lib/rust/crates/awnative/src/clipboard_store.rs'
KOTLIN = ROOT / 'app/src/main/kotlin'
LEGACY_KT = KOTLIN / 'com/jtn0123/audiowhisper/stores/LegacyRoom.kt'
STORE_FILE = KOTLIN / 'dev/patrickgold/florisboard/ime/clipboard/provider/ClipboardStoreFile.kt'
DICTIONARY = KOTLIN / 'dev/patrickgold/florisboard/ime/dictionary/FlorisUserDictionaryDatabase.kt'
CARGO, JVM, DEVICE = 'cargo:stores', 'jvm:stores', 'device:stores'
RUST, ROWS = [CARGO], [CARGO, JVM]
WORDS_KEEP_PRINT = '                self.words.push(word);\n            }\n        }\n        self.imported.push(print);'
CASES = [
    # The frame: magic and kind, checksum, length, version.
    ('kind-ignored', CODEC, 'bytes[..4] != MAGIC || bytes[4..8] != kind {', 'bytes[..4] != MAGIC {', RUST),
    ('checksum-ignored', CODEC, 'if crc32(body).to_le_bytes() != crc {', 'if false {', RUST),
    ('length-ignored', CODEC, 'if length != (body.len() - HEADER) as u64 {', 'if false {', RUST),
    ('newer-read-as-current', CODEC, 'if version > newest {', 'if false {', RUST + [DEVICE]),
    ('crc-not-zlib', CODEC, '0xedb8_8320 ^ (c >> 1)', '0xedb8_8321 ^ (c >> 1)', RUST + [DEVICE]),
    # The payload's pieces.
    ('text-big-endian', CODEC, 'self.bytes.extend_from_slice(&unit.to_le_bytes());',
     'self.bytes.extend_from_slice(&unit.to_be_bytes());', RUST),
    ('trailing-bytes-accepted', CODEC, 'if self.rest.is_empty() {\n            Ok(())', 'if true {\n            Ok(())', RUST),
    ('count-unchecked', CODEC, 'if count.saturating_mul(size) > self.rest.len() {', 'if false && count > 0 {', RUST),
    ('bad-optional-tag-accepted', CODEC, '_ => Err(Error::Damaged("optional text")),', '_ => Ok(None),', RUST),
    ('clip-flags-swapped', CLIP_CODEC, 'writer.u8(flag(clip.pinned, PINNED)\n        | flag(clip.sensitive, SENSITIVE)',
     'writer.u8(flag(clip.pinned, SENSITIVE)\n        | flag(clip.sensitive, PINNED)', RUST),
    ('unknown-clip-flags-accepted', CLIP_CODEC, 'if flags & !(PINNED | SENSITIVE | REMOTE) != 0 {\n        return Err(Error::Damaged("clip flags"));',
     'if false {\n        return Err(Error::Damaged("clip flags"));', RUST),
    ('clip-ids-unordered', CLIP_CODEC, 'if !ordered || items.last().is_some_and(|clip| clip.id >= next_id) {',
     'if items.last().is_some_and(|clip| clip.id >= next_id) {', RUST),
    ('file-orientation-dropped', CLIP_CODEC, 'orientation: reader.i32()?,', 'orientation: reader.i32().map(|_| 0)?,', RUST),
    ('word-locale-and-shortcut-swapped', WORD_CODEC,
     '        locale: reader.optional_text()?,\n        shortcut: reader.optional_text()?,',
     '        shortcut: reader.optional_text()?,\n        locale: reader.optional_text()?,', RUST),
    # Saving and loading the file.
    ('no-temporary-file', DISK, 'disk.write_synced(&temporary, bytes)?;\n    disk.rename(&temporary, path)?;',
     'let _ = &temporary;\n    disk.write_synced(path, bytes)?;', RUST),
    ('rename-not-synced', DISK, '    disk.rename(&temporary, path)?;\n    disk.sync_dir(directory(path))',
     '    disk.rename(&temporary, path)', RUST),
    ('stale-temporary-kept', DISK, '    disk.remove(&temporary(path))?;\n    disk.read(path)', '    disk.read(path)',
     RUST + [DEVICE]),
    ('journal-removed-first', DISK, '    for path in paths {', '    for path in paths.iter().rev() {', RUST),
    ('removal-not-synced', DISK, '        disk.remove(path)?;\n        disk.sync_dir(directory(path))?;',
     '        disk.remove(path)?;', RUST),
    # The store around the value.
    ('damaged-kept-as-found', STORE, 'Err(_) => (T::default(), Opened::Damaged),',
     'Err(_) => (T::default(), Opened::Unreadable),', RUST),
    ('newer-file-overwritten', STORE, 'matches!(self.opened, Opened::Newer | Opened::Unreadable)',
     'matches!(self.opened, Opened::Unreadable)', RUST + [DEVICE]),
    ('kept-before-saved', STORE,
     '            disk::save(self.disk.as_mut(), &self.path, &encode(&next))?;\n            self.value = next;',
     '            self.value = next.clone();\n            disk::save(self.disk.as_mut(), &self.path, &encode(&next))?;',
     RUST),
    ('unchanged-written', STORE, '        if next != self.value {', '        if true {', RUST),
    ('unsaved-commit-kept', STORE, '        if saved.is_err() {\n            self.value = before;\n        }',
     '        let _ = before;', RUST),
    ('rollback-keeps', STORE, '        if let Some(before) = self.before.take() {\n            self.value = before;',
     '        if let Some(before) = self.before.take() {\n            let _ = before;', ROWS),
    ('nested-change-saved', STORE, '        if self.before.is_some() {\n            return Ok(change(&mut self.value));',
     '        if false {\n            return Ok(change(&mut self.value));', ROWS),
    ('verify-trusts-the-write', STORE,
     '        let bytes = self.disk.read(&self.path)?;\n        Ok(bytes.is_some_and(|bytes| decode::<T>(&bytes).is_ok_and(|value| value == self.value)))',
     '        Ok(true)', RUST),
    # The import of main's databases.
    ('kept-store-imports', LEGACY, '    if store.read_only() {\n        return Ok(Imported::Kept);\n    }\n', '', RUST),
    ('imported-twice', LEGACY, 'if store.value().imported().contains(&print) {', 'if false {', RUST + [DEVICE]),
    ('import-unverified', LEGACY, '            if !store.verify()? {', '            if false {', RUST),
    ('files-kept', LEGACY, '    disk::remove_all(store.disk(), files)?;\n', '', RUST + [DEVICE]),
    ('sequence-ignored', LEGACY, 'let largest = ids.max().unwrap_or(0).max(sequence);', 'let largest = ids.max().unwrap_or(0);',
     RUST + [DEVICE]),
    ('import-renumbers', LEGACY, '        if self.items.is_empty() {\n            self.next_id = next_after(',
     '        if false {\n            self.next_id = next_after(', RUST + [DEVICE]),
    ('clip-fingerprint-forgotten', LEGACY,
     '                self.files.push(file);\n            }\n        }\n        self.imported.push(print);',
     '                self.files.push(file);\n            }\n        }\n        let _ = print;', RUST),
    ('word-fingerprint-forgotten', LEGACY, WORDS_KEEP_PRINT, WORDS_KEEP_PRINT.replace('self.imported.push(print);', 'let _ = print;'),
     RUST),
    ('own-files-replaced', LEGACY, '            if self.file(file.id).is_none() {\n                self.files.push(file);',
     '            if true {\n                self.files.push(file);', RUST),
    # The clipboard history's changes.
    ('ids-reused', CLIPS, '        let id = self.next_id.max(1);',
     '        let id = self.items.iter().map(|c| c.id).max().unwrap_or(0) + 1;', ROWS),
    ('copy-again-adds', CLIPS, 'let duplicate = policy::duplicate(&shown, clip.text.as_deref(), true, clip.pinned);',
     'let duplicate: Option<(usize, bool)> = None;', ROWS),
    ('moved-clip-unpinned', CLIPS, '            self.remove(|row| row.id != ids[at]);\n            clip.pinned = pinned;',
     '            self.remove(|row| row.id != ids[at]);\n            clip.pinned = clip.pinned && pinned;', ROWS),
    ('restore-duplicates', CLIPS, 'if !before.iter().any(|row| row.same_as(&clip)) {', 'if true {', ROWS + [DEVICE]),
    ('restore-ignores-limit', CLIPS, '        self.enforce(limit);\n        added', '        added', ROWS),
    ('pin-ignores-limit', CLIPS, '            clip.pinned = pinned;\n        }\n        self.enforce(limit);',
     '            clip.pinned = pinned;\n        }', ROWS),
    ('delete-ignores-pins', CLIPS, '(only_unpinned && clip.pinned)', '(only_unpinned && false)', ROWS),
    ('clear-ignores-pins', CLIPS, 'self.remove(|clip| unpinned_only && clip.pinned);', 'self.remove(|_| false);', ROWS),
    ('orphans-still-used', CLIPS, 'if !used && !uris.iter().any(|known| known == uri) {',
     'if !uris.iter().any(|known| known == uri) && !used.then_some(()).is_some_and(|_| false) {', RUST),
    ('nothing-expires', CLIPS, '        self.remove(|clip| !expired.contains(&clip.id));', '        let _ = expired;', ROWS),
    ('limit-not-enforced', CLIPS, '        if !over.is_empty() {', '        if false {', ROWS),
    ('file-record-doubled', CLIPS, '        self.remove_file(file.id);\n        self.files.push(file);',
     '        self.files.push(file);', RUST),
    # The user dictionary's changes.
    ('zero-id-kept', WORDS, '        if word.id == 0 {\n            word.id = self.next_id.max(1);\n        }\n', '', ROWS),
    ('taken-id-replaced', WORDS, '            Ok(_) => return Err(Taken(word.id)),',
     '            Ok(at) => {\n                self.words.remove(at);\n                at\n            }', ROWS),
    ('next-id-not-advanced', WORDS, '        self.next_id = self.next_id.max(id + 1);', '        self.next_id = self.next_id.max(1);',
     ROWS),
    ('lookup-ignores-locale', WORDS, 'row.word == word && row.locale.as_deref() == locale',
     'row.word == word && locale.is_none_or(|_| true)', ROWS),
    ('update-ignored', WORDS, '        if let Some(row) = self.words.iter_mut().find(|row| row.id == word.id) {\n            *row = word;',
     '        if let Some(row) = self.words.iter_mut().find(|row| row.id == word.id) {\n            let _ = (row, word);', ROWS),
    ('delete-ignored', WORDS, '        self.words.retain(|row| row.id != id);', '        let _ = id;', ROWS),
    # The bridge's rows, and the Kotlin on either side of it.
    ('uri-flag-lost', BRIDGE, 'flags |= if clip.uri.is_some() { HAS_URI } else { 0 };', 'flags |= 0;', [JVM, DEVICE]),
    ('remote-flag-lost', BRIDGE, 'remote: flags & REMOTE != 0,', 'remote: false,', [JVM, DEVICE]),
    ('backup-flags-swapped', STORE_FILE, '(if (isPinned) ClipFlag.PINNED.code else 0) or\n    (if (isSensitive) ClipFlag.SENSITIVE.code else 0)',
     '(if (isPinned) ClipFlag.SENSITIVE.code else 0) or\n    (if (isSensitive) ClipFlag.PINNED.code else 0)', [DEVICE]),
    ('shown-uri-unflagged', STORE_FILE, 'uri.takeIf { has(ClipFlag.HAS_URI) }?.toUri()', 'uri.toUri()', [DEVICE]),
    ('wal-left-behind', LEGACY_KT, 'listOf("-wal", "-shm", "-journal")', 'listOf("-shm", "-journal")', [DEVICE]),
    ('newer-store-databases-opened', LEGACY_KT,
     '        if (paths.none(File::exists)) return null\n        if (store.opened.readOnly) return StoreImport.KEPT\n        val clips',
     '        if (paths.none(File::exists)) return null\n        val clips', [DEVICE]),
    ('legacy-sequence-unread', LEGACY_KT, 'sequence = sequence(db, HISTORY)', 'sequence = 0', [DEVICE]),
    ('old-sensitive-column-unread', LEGACY_KT, 'row.flag("is_sensitive", "isSensitive")', 'row.flag("is_sensitive")', [DEVICE]),
    ('legacy-orientation-unread', LEGACY_KT, 'if (it >= 0) row.getInt(it) else 0', 'if (it >= 0) 0 else 0', [DEVICE]),
    ('damaged-database-kept', LEGACY_KT, 'Log.w(TAG, "${file.name} is damaged: imported as empty")\n                return empty',
     'Log.w(TAG, "${file.name} is damaged: imported as empty")\n                return null', [DEVICE]),
    ('word-sequence-unread', LEGACY_KT, 'sequence = sequence(db, "words")', 'sequence = 0', [DEVICE]),
    ('transaction-not-rolled-back', DICTIONARY, '        } catch (e: Throwable) {\n            store.rollback()',
     '        } catch (e: Throwable) {\n            store.commit()', [JVM]),
    ('lookup-without-locale', DICTIONARY, 'store.exact(word, locale?.localeTag())', 'store.exact(word, null)', [JVM]),
]
JVM_TESTS = ['*ClipboardStoreMatchTest', '*UserWordStoreMatchTest']
DEVICE_TESTS = 'com.jtn0123.audiowhisper.stores.StoresMigrationDeviceTest,com.jtn0123.audiowhisper.stores.StoresBackupDeviceTest'
SERIAL = 'emulator-5556'
PACKAGE = 'com.jtn0123.audiowhisper.keyboard'
RUNNER = PACKAGE + '.test/androidx.test.runner.AndroidJUnitRunner'
IME = PACKAGE + '/dev.patrickgold.florisboard.FlorisImeService'
TEST_APK = ROOT / 'app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk'
REPORTS = ROOT / 'app/build/test-results/testDebugUnitTest'
DEVICE_FAILURE = re.compile(r'^(java\.lang\.AssertionError|junit\.framework\.AssertionFailedError|'
                            r'org\.junit\.ComparisonFailure|org\.junit\.internal\.ArrayComparisonFailure)\b', re.M)


def env():
    e = dict(os.environ)
    e.setdefault('CARGO_BUILD_JOBS', '6')
    if 'ANDROID_HOME' not in e:
        props = (ROOT / 'local.properties').read_text().splitlines()
        e['ANDROID_HOME'] = next(line.split('=', 1)[1] for line in props if line.startswith('sdk.dir='))
    return e


def adb(*args):
    return [str(Path(env()['ANDROID_HOME']) / 'platform-tools/adb'), '-s', SERIAL, *args]


def log_of(output, name, target):
    return output / (name + '.' + target.replace(':', '-') + '.log')


def compiled(log_path):
    """False when the build failed: a mutant that doesn't compile is no test's catch."""
    text = log_path.read_text(errors='replace')
    return not any(sign in text for sign in ('could not compile', '\ne: ', 'Compilation error', 'BUILD FAILED'))


def rust(output, name, target):
    log_path = log_of(output, name, target)
    with log_path.open('wb') as log:
        result = subprocess.run(['cargo', 'test', '-q', '-p', 'stores'], cwd=ROOT / 'lib/rust', stdout=log,
                                stderr=subprocess.STDOUT, env=env())
    text = log_path.read_text(errors='replace')
    return result.returncode, sum('panicked at' in line for line in text.splitlines())


def assertion(failure):
    kind = failure.attrib.get('type', '')
    return 'AssertionFailedError' in kind or kind == 'java.lang.AssertionError'


def jvm(output, name, target):
    started = time.time()
    command = ['./gradlew', '--stacktrace', ':app:testDebugUnitTest']
    for pattern in JVM_TESTS:
        command += ['--tests', pattern]
    with log_of(output, name, target).open('wb') as log:
        result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=env())
    failures = 0
    for report in REPORTS.glob('TEST-*.xml'):
        if report.stat().st_mtime >= started:
            failures += sum(assertion(f) for f in ET.parse(report).getroot().iter('failure'))
    return result.returncode, failures


def device(output, name, target):
    """Builds the fixture app and its tests, installs both, runs the store tests; exit 1 for any failure."""
    with log_of(output, name, target).open('wb') as log:
        steps = [
            (['./gradlew', '--stacktrace', '-PstoresFixture=true', ':app:assembleDebug', ':app:assembleDebugAndroidTest'], {}),
            (['scripts/install-dev.sh', SERIAL], {}),
            (['scripts/install-dev.sh', '--probe', str(TEST_APK)], {'ANDROID_SERIAL': SERIAL}),
        ]
        for command, extra in steps:
            if subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env={**env(), **extra}).returncode:
                return 2, 0
        result = subprocess.run(adb('shell', 'am', 'instrument', '-w', '-e', 'class', DEVICE_TESTS, RUNNER),
                                cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, env=env())
        log.write(result.stdout)
    text = result.stdout.decode(errors='replace')
    passed = re.search(r'^OK \(\d+ tests?\)', text, re.M) is not None
    return (0 if passed else 1), len(DEVICE_FAILURE.findall(text))


def run(output, name, target):
    return {CARGO: rust, JVM: jvm, DEVICE: device}[target](output, name, target)


def check(cases):
    originals = {path: path.read_bytes() for _, path, _, _, _ in cases}
    for name, path, old, new, _ in cases:
        assert originals[path].decode().count(old) == 1, 'mutation location changed: ' + name
        assert old != new, 'mutation changes nothing: ' + name
    names = [case[0] for case in cases]
    assert len(names) == len(set(names)), 'case names repeat'
    return originals


def touch(path):
    """Marks [path] changed for Cargo, which compares whole-second times on some disks (HFS+ keeps no finer):
    a write in the same second as the last build's start would leave the previous mutant's binary in place."""
    time.sleep(1.1)
    os.utime(path)


def mutate(output, cases, originals, targets):
    results = []
    for name, path, old, new, named in cases:
        ran = [t for t in named if t in targets]
        if not ran:
            continue
        mutation_cases.write(path, originals[path].decode().replace(old, new))
        touch(path)
        try:
            outcome = {}
            for target in ran:
                status, failures = run(output, name, target)
                outcome[target] = {'exit': status, 'assertion_failures': failures,
                                   'compiled': compiled(log_of(output, name, target))}
        finally:
            mutation_cases.write(path, originals[path])
            touch(path)
        caught = [t for t, o in outcome.items() if o['exit'] != 0 and o['assertion_failures'] > 0]
        results.append({'mutation': name, 'file': str(path.relative_to(ROOT)), 'targets': outcome, 'caught_by': caught})
        (output / 'result.json').write_text(json.dumps({'mutations': results}, indent=2) + '\n')
        broken = [t for t, o in outcome.items() if not o['compiled']]
        missed = 'NOT CAUGHT' + (' (did not compile for ' + ', '.join(broken) + ')' if broken else '')
        print(name + ': ' + ('caught by ' + ', '.join(caught) if caught else missed), flush=True)
    return results


def check_edits():
    return mutation_cases.rows(CASES)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('cases', nargs='*', help='case names to run (default: all)')
    parser.add_argument('--targets', default=','.join([CARGO, JVM, DEVICE]), help='the targets to run, comma-separated')
    args = parser.parse_args()
    targets = args.targets.split(',')
    assert set(targets) <= {CARGO, JVM, DEVICE}, f'unknown targets: {targets}'
    unknown = set(args.cases) - {case[0] for case in CASES}
    assert not unknown, f'unknown cases: {sorted(unknown)}'
    cases = [case for case in CASES if not args.cases or case[0] in args.cases]
    originals = check(cases)
    output = ROOT / args.output
    output.mkdir(parents=True, exist_ok=False)
    try:
        results = mutate(output, cases, originals, targets)
    finally:
        for path, original in originals.items():
            mutation_cases.write(path, original)
    restored = {}
    for target in sorted({t for case in cases for t in case[4] if t in targets}):
        status, failures = run(output, 'restored', target)
        restored[target] = {'exit': status, 'assertion_failures': failures}
    if DEVICE in targets:
        subprocess.run(adb('shell', 'ime', 'set', IME), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=env())
    missed = [r['mutation'] for r in results if not r['caught_by']]
    report = {'mutations': results, 'missed': missed, 'restored': restored, 'targets': targets,
              'source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in originals}}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    assert all(r['exit'] == 0 and r['assertion_failures'] == 0 for r in restored.values()), 'restored source fails'
    assert not missed, f'not caught: {missed}'
    print(f'All {len(results)} mutations caught; restored source passes {sorted(restored)}', flush=True)


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
