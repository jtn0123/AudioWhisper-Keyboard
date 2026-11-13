"""Fixed subtype/one-handed fixtures. Private settings are restored byte-for-byte.

Only the disposable empty-editor tree and fixed fixture digests are recorded. All adb
input runs with agent-device idle; screenshots use the device_open command explicitly.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import runpy
import shlex
import subprocess
import time

panel = runpy.run_path(str(Path(__file__).with_name('panels.py')))
Device, PKG, PROBE, SERVICE = (panel[k] for k in ('Device','PKG','PROBE','SERVICE'))
PREFS = panel['api']['PREFS']
KEYS = {
    'keyboard__rust_renderer_prototype':'b', 'keyboard__one_handed_mode_enabled':'b',
    'keyboard__one_handed_mode':'s', 'keyboard__one_handed_mode_scale_factor':'i',
    'keyboard__landscape_input_ui_mode':'s', 'keyboard__height_factor_portrait':'i',
    'keyboard__height_factor_landscape':'i', 'keyboard__incognito_indicator':'s',
    'localization__subtypes':'s', 'localization__active_subtype_id':'l',
    'gestures__space_bar_long_press':'s', 'suggestion__incognito_mode':'s',
    'suggestion__enabled':'b','emoji__suggestion_enabled':'b','theme__mode':'s',
    'theme__day_theme_id':'s','theme__night_theme_id':'s',
}


def set_preferences(device, values):
    if not values.keys() <= KEYS.keys():
        raise ValueError('Only declared renderer fixture preferences may be changed')
    raw = device.adb('exec-out','run-as',PKG,'cat',PREFS)
    rows = [row for row in raw.splitlines() if len(p := row.split(';',2)) != 3 or p[1] not in values]
    rows += [KEYS[key]+';'+key+';'+json.dumps(value,ensure_ascii=False,separators=(',',':')) for key,value in values.items()]
    device.shell('am','force-stop',PKG)
    subprocess.run(['adb','-s',device.serial,'shell','run-as '+PKG+' sh -c '+shlex.quote('cat > '+PREFS)],
        input=('\n'.join(rows)+'\n').encode(),check=True)
    device.shell('ime','set',PKG+'/dev.patrickgold.florisboard.FlorisImeService')


def subtypes(original):
    configured = json.loads(json.loads(original.get('localization__subtypes','"[]"')))
    if configured:
        first = configured[0]
    else:
        first = {'id':-1,'primaryLocale':'en-US','secondaryLocales':[],
            'composer':'org.florisboard.composers:appender','currencySet':'org.florisboard.currencysets:dollar',
            'punctuationRule':'org.florisboard.localization:default','popupMapping':'org.florisboard.localization:en',
            'layoutMap':{'characters':'org.florisboard.layouts:qwerty'}}
    second = dict(first, id=8008, primaryLocale='en-GB')
    return [first,second]


def prepare(device, renderer, original, theme='day', side=None, incognito=False):
    values={'keyboard__rust_renderer_prototype':renderer=='rust',
        'localization__subtypes':json.dumps(subtypes(original),ensure_ascii=False,separators=(',',':')),
        'localization__active_subtype_id':subtypes(original)[0]['id'],
        'keyboard__one_handed_mode_enabled':side is not None,
        'keyboard__one_handed_mode':side or 'END','keyboard__one_handed_mode_scale_factor':80,
        'suggestion__enabled':False,'emoji__suggestion_enabled':False,
        'suggestion__incognito_mode':'FORCE_ON' if incognito else 'FORCE_OFF',
        'keyboard__incognito_indicator':'DISPLAY_BEHIND_KEYBOARD',
        'gestures__space_bar_long_press':'SHOW_SUBTYPE_PICKER',
        'theme__mode':'ALWAYS_'+theme.upper(),
        'theme__'+theme+'_theme_id':'org.florisboard.themes:floris_'+theme,
        'keyboard__landscape_input_ui_mode':'ALWAYS_SHOW'}
    set_preferences(device,values)
    device.probe(True);device.services(SERVICE)
    data=device.capture('ready',lambda ns:any(n.get('text') in ('e','E') for n in ns))
    if any(n.get('text')=='E' for n in data['nodes']):
        panel['tap_name'](device,'Shift',data['nodes'])
        data=device.capture('lowercase',lambda ns:any(n.get('text')=='e' for n in ns))
    return data


def sheet(device):
    # Invoke the real configured space hold handler, with no direct state injection.
    nodes=device.tree()['nodes']
    space=next((n for n in nodes if n.get('description')=='Space'),None)
    point=panel['center'](space) if space else (540,2200)
    device.hold(*point,1100)
    return device.capture('subtype-sheet',lambda ns:any(n.get('text')=='Select subtype' for n in ns))


def capture(device,agent,name):
    device.capture(name)
    panel['screenshot'](device,agent,name)


def workflow(device, original):
    results={};root=device.output
    for renderer in ('compose','rust'):
        device.output=root/renderer;device.output.mkdir(parents=True,exist_ok=True)
        prepare(device,renderer,original);sheet(device)
        panel['tap_name'](device,'English (United Kingdom)')
        device.wait(lambda:json.loads(device.prefs()['localization__active_subtype_id'])==8008,'Subtype selection did not persist')
        selected={k:v for k,v in device.prefs().items() if k in ('localization__subtypes','localization__active_subtype_id')}
        prepare(device,renderer,original,side='START')
        points=[panel['center'](panel['named'](device.tree()['nodes'],key)) for key in 'cat']
        panel['observer_off'](device)
        for point in points:device.tap(*point)
        digest=panel['digest'](device,'cat','one-handed')
        stored={k:v for k,v in device.prefs().items() if k.startswith('keyboard__one_handed')}
        results[renderer]={'subtype':selected,'one_handed':stored,'digest':digest}
        device.reset(True)
    device.output=root
    device.save('comparison.json',{'match':results['compose']==results['rust'],'results':results})
    assert results['compose']==results['rust']


def visuals(device,original,agent):
    root=device.output
    names=('subtypes','onehand-left','onehand-right','incognito','extract')
    for theme in ('day','night'):
        for renderer in ('compose','rust'):
            device.output=root/(theme+'-'+renderer);device.output.mkdir(parents=True,exist_ok=True)
            prepare(device,renderer,original,theme);sheet(device);capture(device,agent,'subtypes')
            device.tap(540,1300)
            for side,name in (('START','onehand-left'),('END','onehand-right')):
                prepare(device,renderer,original,theme,side);capture(device,agent,name)
            prepare(device,renderer,original,theme,incognito=True);capture(device,agent,'incognito')
            device.shell('settings','put','system','accelerometer_rotation',0)
            device.shell('settings','put','system','user_rotation',1)
            time.sleep(1)
            prepare(device,renderer,original,theme);capture(device,agent,'extract')
            device.shell('settings','put','system','user_rotation',0);time.sleep(1)
        for name in names:
            subprocess.check_call(['swift','scripts/render-benchmark/chrome-pair.swift',
                str(root/(theme+'-compose')/(name+'.png')),str(root/(theme+'-rust')/(name+'.png')),
                str(root/(theme+'-'+name+'-pair.png'))])
    device.output=root


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('mode',choices=('workflow','visuals'));parser.add_argument('output');parser.add_argument('--agent')
    args=parser.parse_args();device=Device(args.output)
    original=device.prefs();accessibility=device.original_accessibility()
    rotation={k:device.shell('settings','get','system',k).strip() for k in ('accelerometer_rotation','user_rotation')}
    private=device.output/'preferences.private'
    private.write_bytes(subprocess.check_output(['adb','-s',device.serial,'exec-out','run-as',PKG,'cat',PREFS]))
    try:
        if args.mode=='workflow':workflow(device,original)
        else:
            if not args.agent:raise ValueError('device_open agent command required')
            visuals(device,original,args.agent)
    finally:
        device.shell('am','force-stop',PKG)
        subprocess.run(['adb','-s',device.serial,'shell','run-as '+PKG+' sh -c '+shlex.quote('cat > '+PREFS)],input=private.read_bytes(),check=True)
        for key,value in rotation.items():
            device.shell('settings','delete' if value=='null' else 'put','system',key,*([] if value=='null' else [value]))
        device.restore_accessibility(accessibility)
        device.shell('ime','set',PKG+'/dev.patrickgold.florisboard.FlorisImeService')


if __name__=='__main__':main()
