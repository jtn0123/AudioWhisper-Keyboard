"""Mutate production decisions, require assertion/parity failures, and restore exact source bytes.

python3 -I scripts/dictation-mutations.py OUTPUT [state|device]
Device mode requires owned ANDROID_SERIAL and the generated private fixture WAVs from a passing run.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
os.chdir(ROOT)
out = Path(sys.argv[1]).resolve(); out.mkdir(parents=True, exist_ok=True)
mode = sys.argv[2] if len(sys.argv)>2 else 'state'
PKG = 'com.jtn0123.audiowhisper.keyboard'
serial = os.environ.get('ANDROID_SERIAL', '')
if mode == 'device' and (not serial or serial == 'emulator-5554'):
    raise ValueError('Owned runtime serial required')

def run(argv, name):
    with (out / (name+'.log')).open('wb') as log:
        result = subprocess.run(argv, stdout=log, stderr=subprocess.STDOUT)
    (out / (name+'.exit')).write_text(str(result.returncode)+'\n')
    return result.returncode


def jvm(name):
    reports=ROOT/'app/build/test-results/testDebugUnitTest'
    for report in reports.glob('TEST-*DictationSession*Test.xml'):
        report.unlink()
    status=run(['./gradlew',':app:testDebugUnitTest','--tests','*DictationSession*Test','--tests','*ModelHolderTest'],name)
    failures=0
    for path in (ROOT/'app/build/test-results/testDebugUnitTest').glob('TEST-*DictationSession*Test.xml'):
        node=ET.parse(path).getroot()
        for failure in node.findall('.//failure'):
            assert failure.get('type') in ('org.opentest4j.AssertionFailedError','java.lang.AssertionError'), 'Not an assertion failure'
        failures+=int(node.get('failures','0'))
        (out/(name+'-'+path.name)).write_bytes(path.read_bytes())
    return status,failures


def device(name):
    assert run(['./gradlew',':app:assembleDebug',':app:assembleDebugAndroidTest','-PdictationFixture=true'],name+'-build')==0
    assert run(['scripts/install-dev.sh'],name+'-install')==0
    assert run(['scripts/install-dev.sh','--probe','app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk'],name+'-test-install')==0
    subprocess.run(['adb','-s',serial,'shell','run-as',PKG,'rm','-f','files/dictation-comparison.json'],check=True)
    assert run(['adb','-s',serial,'shell','am','instrument','-w','-e','reuse','true','-e','timings','false',
                PKG+'.test/com.jtn0123.audiowhisper.dictation.DictationInstrumentation'],name+'-comparison')==0
    raw=subprocess.check_output(['adb','-s',serial,'exec-out','run-as',PKG,'cat','files/dictation-comparison.json'])
    report=json.loads(raw);(out/(name+'-report.json')).write_bytes(raw)
    return report

cases = [
 ('timeout','lib/rust/crates/dictation/src/session.rs','KEEP_LOADED_MS: i64 = 300_000','KEEP_LOADED_MS: i64 = 300_001'),
 ('late-result','lib/rust/crates/dictation/src/session.rs','events::FINAL if id == self.generation','events::FINAL if id <= self.generation'),
] if mode=='state' else [
 ('threshold','lib/rust/crates/dictation/src/segmentation.rs','VAD_THRESHOLD: f32 = 0.5','VAD_THRESHOLD: f32 = 0.99'),
 ('tail-flush','lib/rust/crates/sherpa-ffi/src/vad.rs','if keep {','if !keep {'),
]
results=[]
for name,relative,before,after in cases:
    path=ROOT/relative; original=path.read_bytes();source=original.decode()
    assert source.count(before)==1,(name,source.count(before))
    try:
        path.write_text(source.replace(before,after))
        if mode=='state':
            status,failures=jvm(name)
            assert status!=0 and failures>0, 'Mutation was not caught by JVM assertions'
            results.append({'mutation':name,'gradle_exit':status,'assertions':failures})
        else:
            report=device(name)
            assert report.get('identical') is False and report.get('failure_kind')=='parity','Mutation was not caught by audio parity'
            results.append({'mutation':name,'parity_failure':True,'fixture':report['fixture']})
    finally:
        path.write_bytes(original)
    assert path.read_bytes()==original
if mode=='state':
    status,failures=jvm('restored');assert status==0 and failures==0
else:
    assert device('restored')['identical'] is True
(out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps(results))
