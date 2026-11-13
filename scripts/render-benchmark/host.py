"""Opt-in timestamp-only input-view factory/first-layout measurements, 10 cold + 10 warm.

Cold kills only our keyboard process. Warm recreates the view through portrait/landscape
configuration changes in the same process. These measure the complete current paths,
not an unimplemented plain-View alternative or physical presentation latency.
"""
import argparse
import json
import re
import runpy
import statistics
import time
from pathlib import Path

chrome=runpy.run_path(str(Path(__file__).with_name('chrome.py')))
Device=chrome['Device'];PKG=chrome['PKG']


def timing(device):
    text=device.adb('logcat','-d','-s','aw-host:D')
    starts={}
    for renderer,start,key,value in re.findall(r'renderer=(\d) start_ns=(\d+) (created|layout)_ns=(\d+)',text):
        starts.setdefault((int(renderer),int(start)),{})[key]=int(value)
    ready=[{'renderer':r,'start_ns':s,**values} for (r,s),values in starts.items() if 'created' in values and 'layout' in values]
    return ready[-1] if ready else None


def main():
    parser=argparse.ArgumentParser();parser.add_argument('output');args=parser.parse_args()
    d=Device(args.output);original=d.prefs();accessibility=d.original_accessibility()
    rotation={k:d.shell('settings','get','system',k).strip() for k in ('accelerometer_rotation','user_rotation')}
    results={}
    try:
        d.shell('setprop','log.tag.aw-host','DEBUG')
        d.shell('settings','put','system','accelerometer_rotation',0)
        d.shell('settings','put','system','user_rotation',0)
        for renderer in ('compose','rust'):
            chrome['set_preferences'](d,{'keyboard__rust_renderer_prototype':renderer=='rust'})
            runs={'cold':[],'warm':[]}
            for _ in range(10):
                d.shell('am','force-stop',PKG);d.adb('logcat','-c');d.probe(True)
                item=d.wait(lambda:timing(d),'Cold input-view timestamps absent')
                assert item['renderer']==int(renderer=='rust');item['pid']=d.shell('pidof',PKG).strip();runs['cold'].append(item)
            for _ in range(10):
                d.shell('settings','put','system','user_rotation',1)
                time.sleep(.8)
                d.adb('logcat','-c')
                d.shell('settings','put','system','user_rotation',0)
                item=d.wait(lambda:timing(d),'Warm input-view timestamps absent')
                assert item['renderer']==int(renderer=='rust');item['pid']=d.shell('pidof',PKG).strip();runs['warm'].append(item)
            results[renderer]=runs;d.save('runs.json',results)
        summary={renderer:{kind:{metric:{'median_ms':statistics.median((x[metric]-x['start_ns'])/1e6 for x in cohort),
            'samples_ms':[(x[metric]-x['start_ns'])/1e6 for x in cohort]} for metric in ('created','layout')}
            for kind,cohort in runs.items()} for renderer,runs in results.items()}
        d.save('summary.json',summary)
    finally:
        d.shell('setprop','log.tag.aw-host','INFO')
        for key,value in rotation.items():
            d.shell('settings','delete' if value=='null' else 'put','system',key,*([] if value=='null' else [value]))
        chrome['set_preferences'](d,{'keyboard__rust_renderer_prototype':json.loads(original.get('keyboard__rust_renderer_prototype','false'))})
        d.restore_accessibility(accessibility)


if __name__=='__main__':main()
