"""Capture UI states from a shell-only source set, absent from normal APKs.

python3 -I scripts/dictation-ui.py OUTPUT reference|native AGENT_DEVICE_COMMAND
Install the matching -PdictationFixture=true APK. No instrumentation restarts its IME process.
"""
import json
import os
from pathlib import Path
import runpy
import subprocess
import sys
import time

root=Path(__file__).resolve().parents[1];os.chdir(root)
api=runpy.run_path('scripts/render-benchmark/a11y_device.py')
chrome=runpy.run_path('scripts/render-benchmark/chrome.py')
panels=runpy.run_path('scripts/render-benchmark/panels.py')
d=api['Device'](sys.argv[1]);reference=sys.argv[2];agent=sys.argv[3]
assert reference in ('reference','native')
original=d.original_accessibility();d.save('original-preferences.json',d.prefs())
held=False
model='files/models/sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8/encoder.int8.onnx'
permission='android.permission.RECORD_AUDIO'
assert permission+': granted=true' in d.shell('dumpsys','package',api['PKG'])
def state(op):
    return d.shell('am','broadcast','-a','com.jtn0123.audiowhisper.DICTATION_UI','-n',api['PKG']+'/com.jtn0123.audiowhisper.dictation.DictationUiReceiver','--es','op',op)
try:
    d.mute();subprocess.run([agent,'open',api['PROBE'],'--no-test-ime'],check=True,stdout=subprocess.DEVNULL)
    chrome['set_preferences'](d,{'keyboard__rust_renderer_prototype':True,'theme__mode':'ALWAYS_DAY',
        'theme__day_theme_id':'org.florisboard.themes:floris_day','suggestion__enabled':False,'emoji__suggestion_enabled':False})
    results={}
    for op in ('listening','processing','no-model','no-permission'):
        if op=='no-model':
            assert subprocess.run(['adb','-s',d.serial,'shell','run-as',api['PKG'],'test','-e',model+'.fixture-held']).returncode!=0
            d.shell('run-as',api['PKG'],'mv',model,model+'.fixture-held');held=True
        if op=='no-permission':d.shell('pm','revoke',api['PKG'],permission)
        d.probe(no_suggestions=True);d.services(api['SERVICE'])
        tree=d.capture('native-ready-'+op,lambda ns:any(n.get('visible') and n.get('text') in ('e','E') for n in ns))
        assert sum(bool(n.get('visible') and n.get('textEntryKey')) for n in tree['nodes'])>0
        panels['observer_off'](d)
        d.shell('run-as',api['PKG'],'rm','-f','files/dictation-ui.json')
        state(op)
        def settled():
            try:value=json.loads(d.adb('exec-out','run-as',api['PKG'],'cat','files/dictation-ui.json'))
            except (json.JSONDecodeError,subprocess.CalledProcessError):return None
            return value if value['state']==op else None
        results[op]=d.wait(settled,'UI state absent')
        time.sleep(1)
        panels['screenshot'](d,agent,op)
        state('reset')
        if held:d.shell('run-as',api['PKG'],'mv',model+'.fixture-held',model);held=False
        if op=='no-permission':d.shell('pm','grant',api['PKG'],permission)
    d.save('ui.json',results)
    assert results['listening']['status']=='LISTENING' and results['processing']['status']=='FINISHING'
    assert results['no-model']['status']==results['no-permission']['status']=='IDLE'
    assert results['no-model']['problem'] and results['no-permission']['problem']
    print('Captured four UI states on the '+reference+' APK')
finally:
    state('reset')
    if held:d.shell('run-as',api['PKG'],'mv',model+'.fixture-held',model)
    d.shell('pm','grant',api['PKG'],permission)
    panels['restore_preferences'](d,d.output/'original-preferences.json',list(chrome['KEYS']))
    d.restore_accessibility(original)
    assert permission+': granted=true' in d.shell('dumpsys','package',api['PKG'])
    assert d.shell('run-as',api['PKG'],'stat','-c','%s',model).strip()=='652282309'
