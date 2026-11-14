"""Back up, break and restore actual consolidation transports; compilation errors do not count."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent
MUTATIONS = {
    'learner': ('lib/rust/crates/awnative/src/suggest/word_learner.rs',
                '?.probability(word)', '?.probability(&word[..0])',
                [':app:testDebugUnitTest', '--tests', '*BoundaryLearnerMatchTest']),
    'glide': ('lib/rust/crates/awnative/src/suggest/native_glide.rs',
              'pair_count,', 'pair_count: pair_count.saturating_mul(0),',
              [':app:testDebugUnitTest', '--tests', '*BoundaryGlideMatchTest']),
    'reader': ('lib/rust/crates/awnative/src/reader.rs',
               'index(it.first),', 'index(it.first.saturating_add(1)),',
               [':lib:rust:testDebugUnitTest', '--tests', '*ReaderRulesTest']),
}


def run(output, name, arguments):
    with (output / (name + '.log')).open('wb') as log:
        result = subprocess.run(['./gradlew', '--stacktrace', *arguments], cwd=ROOT,
                                stdout=log, stderr=subprocess.STDOUT)
    return result.returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('item', choices=MUTATIONS)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    relative, before, after, tests = MUTATIONS[args.item]
    path = ROOT / relative
    original = path.read_bytes()
    (args.output / 'original.source').write_bytes(original)
    text = original.decode()
    if text.count(before) != 1:
        raise RuntimeError('Mutation target must be unique')
    reports = ROOT / ('lib/rust' if args.item == 'reader' else 'app') / 'build/test-results/testDebugUnitTest'
    try:
        if reports.exists():
            shutil.rmtree(reports)
        path.write_text(text.replace(before, after))
        status = run(args.output, 'mutant', tests)
        assertions = sum('AssertionFailedError' in failure.get('type', '')
                         or 'AssertionError' in failure.get('type', '')
                         for report in reports.glob('TEST-*.xml')
                         for failure in ET.parse(report).iter('failure'))
        if status == 0 or assertions == 0:
            raise RuntimeError('Mutation did not produce a fresh assertion failure')
        record = {'item': args.item, 'mutant_exit': status, 'assertions': assertions}
    finally:
        path.write_bytes(original)
        if path.read_bytes() != original:
            raise RuntimeError('Byte restoration failed')
    record['restored_sha256'] = hashlib.sha256(path.read_bytes()).hexdigest()
    record['restored_exit'] = run(args.output, 'restored', tests)
    (args.output / 'result.json').write_text(json.dumps(record, indent=2) + '\n')
    if record['restored_exit']:
        raise RuntimeError('Restored comparison failed')
    print(json.dumps(record))


if __name__ == '__main__':
    main()
