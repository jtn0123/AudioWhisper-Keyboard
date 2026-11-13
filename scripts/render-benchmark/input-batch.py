"""Check actual InputConnection batch depth with the test-only instrumentation active."""
import json
from pathlib import Path
import runpy
import sys
import subprocess
import time

api=runpy.run_path(str(Path(__file__).with_name('a11y_device.py')))
d=api['Device'](sys.argv[1])
expected=int(sys.argv[2])
original=d.original_accessibility()
results={}
try:
    for renderer in ('compose','rust'):
        d.renderer(renderer)
        d.probe(True,field="batch")
        before=d.digest()
        assert before['batch_depth']==0
        d.shell('run-as',api['PKG'],'rm','-f','files/render-batch-fixture.json')
        d.shell('am','broadcast','-a','com.jtn0123.audiowhisper.RENDER_FIXTURE','-p',api['PKG'],'--es','op','invalid-finalize')
        def reply():
            result=subprocess.run(['adb','-s',d.serial,'exec-out','run-as',api['PKG'],'cat','files/render-batch-fixture.json'],capture_output=True,text=True)
            if result.returncode != 0:
                return False
            try:
                return json.loads(result.stdout).get('accepted') is False
            except json.JSONDecodeError:
                return False
        d.wait(reply,'Invalid finalize reply missing')
        time.sleep(.3)
        value=d.digest()
        assert value['batch_depth']==expected,'Invalid finalize batch depth mismatch'
        assert value['sha256']==before['sha256'] and value['selection_start']==before['selection_start'] and value['selection_end']==before['selection_end']
        results[renderer]=value
        d.probe(True,field="batch")  # Replace the deliberately leaked baseline connection.
    d.save('batch.json',results)
    print('Both renderers: invalid finalize accepted=false batch_depth='+str(expected))
finally:d.restore_accessibility(original)
