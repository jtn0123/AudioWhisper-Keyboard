"""Compare the existing public glide/candidate/cancel traces with agent-device idle."""
import json
from pathlib import Path
import runpy
import sys

api=runpy.run_path(str(Path(__file__).with_name('a11y_device.py')))
panels=runpy.run_path(str(Path(__file__).with_name('panels.py')))
d=api['Device'](sys.argv[1])
root=d.output
original=d.original_accessibility()
results={}
try:
    d.mute()
    d.probe()
    for renderer in ('compose','rust'):
        d.renderer(renderer)
        d.probe()
        d.services(api['SERVICE'])
        tree=d.capture('ready-'+renderer,lambda ns:any(n.get('text') in ('e','E') for n in ns))
        if any(n.get('text')=='E' for n in tree['nodes']):d.tap(70,2060)
        panels['observer_off'](d)
        saved=sys.argv
        sys.argv=['gestures.py',str(Path(__file__).with_name('adb-input.sh').resolve()),str(root/renderer),'--glide-only']
        try:runpy.run_path(str(Path(__file__).with_name('gestures.py')),run_name='__main__')
        finally:sys.argv=saved
        results[renderer]=json.loads((root/renderer/'gestures.json').read_text())
    assert results['compose']==results['rust'],'Glide renderer state differs'
    d.save('glide.json',results)
    if len(sys.argv)>2:
        baseline=json.loads(Path(sys.argv[2]).read_text())
        assert results.keys()==baseline.keys(),'Glide renderer cohorts differ'
        for renderer,steps in baseline.items():
            assert len(results[renderer])==len(steps),'Glide checkpoint count differs'
            for actual,expected in zip(results[renderer],steps):
                assert all(actual[key]==value for key,value in expected.items()),'Pre/post glide state differs'
    print('Both renderers match existing glide/candidate/cancel digest checkpoints')
finally:d.restore_accessibility(original)
