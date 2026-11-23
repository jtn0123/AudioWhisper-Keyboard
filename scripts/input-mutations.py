"""Run restored input mutations through actual JVM reference comparisons. Use python3 -I.

Run with no other Gradle operation using this checkout. Evidence stays in ignored build/.
"""
import json
from pathlib import Path
import subprocess
import sys
import time
import xml.etree.ElementTree as ET

sys.dont_write_bytecode=True  # importing mutation_cases would otherwise leave a __pycache__ in scripts/
sys.path.insert(0,str(Path(__file__).resolve().parent))
import mutation_cases

ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'lib/rust/crates/input/src'
# label: (file under SOURCE, old, new).
CASES=[
    ('caps-lock-window','lifecycle.rs','                    v[2],\n                )),','                    v[2] + 1,\n                )),'),
    ('repeat-delete-start','lifecycle.rs','    (delay as f32 * factor) as i64\n}',
     '    (delay as f32 * factor) as i64 - i64::from(code == DELETE)\n}'),
    ('symbols-routing','routing.rs','VIEW_SYMBOLS => set!(0, 2),','VIEW_SYMBOLS => set!(0, 3),'),
    ('rolling-multi-pointer','pointer.rs','    suitable(code)\n}', '    suitable(code) && code < 0\n}'),
]
# The delay expression also occurs in repeat_delay. Only the first occurrence is the start delay.
COUNTS={'repeat-delete-start':2}


def check_edits():
    return [mutation_cases.Edit(label,SOURCE/file,old,new,COUNTS.get(label,1)) for label,file,old,new in CASES]


def main():
    output=ROOT/sys.argv[1]
    output.mkdir(parents=True,exist_ok=True)
    edits=CASES
    if len(sys.argv)>2:
        edits=[edit for edit in edits if edit[0]==sys.argv[2]]
        assert edits,'Unknown mutation selector'
    results=[]
    for label,file,old,new in edits:
        path=SOURCE/file
        original=path.read_bytes()
        source=original.decode()
        expected=COUNTS.get(label,1)
        assert source.count(old)==expected,'Mutation location changed: '+label
        try:
            mutation_cases.write(path, source.replace(old,new,1))
            started=time.time()
            with (output/(label+'.log')).open('wb') as log:
                run=subprocess.run(['./gradlew','--stacktrace',':app:testDebugUnitTest','--tests','*Input*MatchTest','--tests','*RendererTouchMatchTest'],
                    cwd=ROOT,stdout=log,stderr=subprocess.STDOUT)
            failures=[]
            for report in (ROOT/'app/build/test-results/testDebugUnitTest').glob('TEST-*.xml'):
                if not ('Input' in report.name or 'RendererTouchMatchTest' in report.name):
                    continue
                if report.stat().st_mtime>=started:
                    failures += [f.attrib.get('type','') for f in ET.parse(report).getroot().iter('failure')]
            caught=run.returncode!=0 and any('AssertionFailedError' in f for f in failures)
            results.append({'mutation':label,'gradle_exit':run.returncode,
                'assertion_failures':sum('AssertionFailedError' in f for f in failures),'caught':caught,'restored':True})
            (output/'mutations.json').write_text(json.dumps(results,indent=2)+'\n')
            print(label+': '+('caught' if caught else 'NOT CAUGHT'),flush=True)
            assert caught,'Mutation did not fail a reference assertion: '+label
        finally:mutation_cases.write(path, original)
    with (output/'restored.log').open('wb') as log:
        run=subprocess.run(['./gradlew','--stacktrace',':app:testDebugUnitTest','--tests','*Input*MatchTest',
            '--tests','*RendererTouchMatchTest','--tests','*Editor*Test'],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT)
    assert run.returncode==0,'Restored tests failed'
    print('All input mutations restored; match tests pass',flush=True)


if __name__=='__main__':
    mutation_cases.check_flag()
    main()
