"""Fixed harmless clipboard and no-key GIF fixtures. No personal clipboard/editor text is recorded."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import runpy
import sqlite3
import subprocess
import time

panels=runpy.run_path(str(Path(__file__).with_name('panels.py')))
visuals=runpy.run_path(str(Path(__file__).with_name('visuals.py')))
Device=panels['Device'];SERVICE=panels['SERVICE'];PKG=panels['PKG'];PROBE=panels['PROBE']
FIXTURES={f'aw-test-{n}' for n in range(1,5)}


def history(device):
    """Copy Room files privately; publish only digests and fixed fixture metadata."""
    folder=device.output/'private-room';folder.mkdir(exist_ok=True)
    for suffix in ('','-wal','-shm'):
        path='databases/clipboard_history'+suffix
        result=subprocess.run(['adb','-s',device.serial,'exec-out','run-as',PKG,'cat',path],capture_output=True)
        file=folder/('clipboard_history'+suffix)
        if result.returncode==0:file.write_bytes(result.stdout)
        elif file.exists():file.unlink()
    db=sqlite3.connect('file:'+str((folder/'clipboard_history').resolve())+'?mode=ro',uri=True)
    try:rows=db.execute('SELECT text,type,isPinned,mimeTypes FROM clipboard_history ORDER BY creationTimestampMs DESC,_id DESC').fetchall()
    finally:db.close()
    if any(row[0] not in FIXTURES for row in rows):raise RuntimeError('History contains non-fixture items; no content was logged')
    return [{'sha256':hashlib.sha256(text.encode()).hexdigest(),'type':kind,'pinned':bool(pin),'mime':mime} for text,kind,pin,mime in rows]


def clip(device,n=None):
    args=['am','broadcast','-a',PROBE+'.CLIP','-p',PROBE]
    if n is not None:args+=['--es','fixture',f'aw-test-{n}']
    device.shell(*args);time.sleep(.3)


def open_clipboard(device):
    data=device.tree()
    if any(n.get('text')=='Clipboard' and not n.get('clickable') for n in data['nodes']):return data
    try:panels['tap_name'](device,'Clipboard',data['nodes'])
    except RuntimeError:
        panels['overflow'](device);panels['tap_name'](device,'Clipboard')
    return device.capture('clipboard-open')


def menu(device,n):
    data=device.tree();node=panels['named'](data['nodes'],f'aw-test-{n}')
    device.hold(*panels['center'](node),700)
    return device.capture(f'clipboard-menu-{n}',lambda ns:any((x.get('text') or x.get('description')) in ('Pin item','Unpin item') for x in ns))


def menu_action(device,n,label):
    data=menu(device,n)
    # The toolbar also has Delete. The popup action is in the right half below the header.
    nodes=[x for x in data['nodes'] if label in (x.get('text'),x.get('description')) and x['bounds'][1]>1650]
    if label=='Delete' and nodes:device.tap(*panels['center'](nodes[-1]))
    else:panels['tap_name'](device,label,data['nodes'])
    time.sleep(.35)


def clear_dialog(device):
    data=device.tree()
    # Both renderers have the same toolbar position; Compose's icons have no spoken label.
    device.tap(814,1618)
    device.capture('clear-dialog-state',lambda ns:any((n.get('text')or n.get('description'))=='Yes' for n in ns))


def clear(device):
    clear_dialog(device)
    panels['tap_name'](device,'Yes');time.sleep(.35)


def prepare(device,renderer):
    device.renderer(renderer)
    device.toggle_preference('clipboard','Enable clipboard history','clipboard__history_enabled',True,default=False)
    device.probe(True);device.services(SERVICE)
    if history(device):raise RuntimeError('Fixture must start with empty history')
    open_clipboard(device)


def seed(device):
    # The disposable probe is foreground and performs Android clipboard writes.
    for n in range(1,5):clip(device,n)
    device.wait(lambda:len(history(device))==4,'Clipboard listener did not store four fixtures')
    menu_action(device,4,'Pin item')
    device.wait(lambda:sum(x['pinned'] for x in history(device))==1,'Pin did not persist')


def cleanup(device):
    state=history(device)
    for n in range(1,5):
        digest=hashlib.sha256(f'aw-test-{n}'.encode()).hexdigest()
        if any(x['sha256']==digest and x['pinned'] for x in state):menu_action(device,n,'Delete')
    if history(device):clear(device)
    clip(device)
    device.wait(lambda:not history(device),'Fixture history cleanup failed')


def workflow(device,renderer):
    prepare(device,renderer);seed(device)
    node=panels['named'](device.tree()['nodes'],'aw-test-2')
    point=panels['center'](node)
    panels['observer_off'](device);device.tap(*point)
    expected=hashlib.sha256(b'aw-test-2').hexdigest()
    device.wait(lambda:device.digest()['sha256']==expected,'History paste committed wrong digest')
    results={'paste':device.digest()}
    device.reset(True);device.services(SERVICE);open_clipboard(device)
    menu_action(device,3,'Pin item');results['pin']=history(device)
    menu_action(device,3,'Unpin item');results['unpin']=history(device)
    menu_action(device,1,'Delete');results['delete']=history(device)
    clear(device);results['clear']=history(device)
    assert len(results['clear'])==1 and results['clear'][0]['pinned']
    device.save('workflow.json',results);cleanup(device)
    return results


def capture(device,agent,name):
    device.capture(name)
    # agent-device is idle during all adb input and semantic capture.
    panels['screenshot'](device,agent,name)


def screenshots(device,agent):
    root=device.output
    for theme in ('day','night'):
        for renderer in ('compose','rust'):
            device.output=root/(theme+'-'+renderer);device.output.mkdir(parents=True,exist_ok=True)
            prepare(device,renderer);visuals['theme'](device,theme);device.probe(True);device.services(SERVICE);open_clipboard(device)
            capture(device,agent,'clipboard-empty');seed(device);capture(device,agent,'clipboard-items')
            menu(device,2);capture(device,agent,'clipboard-menu')
            device.tap(540,2200);clear_dialog(device);capture(device,agent,'clipboard-clear-dialog')
            panels['tap_name'](device,'No');cleanup(device)
            device.probe(True);device.services(SERVICE);panels['tap_name'](device,'Emoji')
            capture(device,agent,'gif-no-key')
        for name in ('clipboard-empty','clipboard-items','clipboard-menu','clipboard-clear-dialog','gif-no-key'):
            paths=[str(root/(theme+'-'+r)/(name+'.png')) for r in ('compose','rust')]
            subprocess.check_call(['swift','scripts/render-benchmark/panel-pair.swift',*paths,str(root/(theme+'-'+name+'-pair.png'))])
    device.output=root


def dialog_screenshots(device,agent):
    root=device.output
    for theme in ('day','night'):
        for renderer in ('compose','rust'):
            device.output=root/(theme+'-'+renderer)
            prepare(device,renderer);visuals['theme'](device,theme);device.probe(True);device.services(SERVICE);open_clipboard(device)
            seed(device);clear_dialog(device);capture(device,agent,'clipboard-clear-dialog')
            panels['tap_name'](device,'No');cleanup(device)
        paths=[str(root/(theme+'-'+r)/'clipboard-clear-dialog.png') for r in ('compose','rust')]
        subprocess.check_call(['swift','scripts/render-benchmark/panel-pair.swift',*paths,str(root/(theme+'-clipboard-clear-dialog-pair.png'))])
    device.output=root


def search(device,renderer,agent=None):
    device.renderer(renderer);device.probe(True);device.services(SERVICE)
    device.shell('am','broadcast','-a','com.jtn0123.audiowhisper.RENDER_FIXTURE','-p',PKG,'--es','op','search')
    device.capture('gif-search-empty',lambda ns:any('Search GIPHY' in (n.get('text') or '') for n in ns))
    if agent:capture(device,agent,'gif-search-empty')
    # Record geometry while the editor is empty, then stop the observer during typing.
    tree=device.tree()['nodes'];points=[panels['center'](panels['named'](tree,ch)) for ch in 'cat']
    panels['observer_off'](device)
    for point in points:device.tap(*point)
    device.shell('am','broadcast','-a','com.jtn0123.audiowhisper.RENDER_FIXTURE','-p',PKG,'--es','op','query')
    time.sleep(.3)
    query=json.loads(device.adb('exec-out','run-as',PKG,'cat','files/render-query-fixture.json'))
    assert query['sha256']==hashlib.sha256(b'cat').hexdigest() and query['typing'] and not query['key_available']
    assert device.digest()['utf16_units']==0
    device.save('gif-search-query.json',query);device.save('gif-search-editor.json',device.digest())
    if agent:
        device.services(SERVICE);capture(device,agent,'gif-search-typing')
    device.shell('am','broadcast','-a','com.jtn0123.audiowhisper.RENDER_FIXTURE','-p',PKG,'--es','op','stop')
    return query


def main():
    parser=argparse.ArgumentParser();parser.add_argument('mode',choices=['workflow','visuals','search','search-visuals','dialogs']);parser.add_argument('output');parser.add_argument('--agent')
    args=parser.parse_args();device=Device(args.output)
    original=device.original_accessibility()
    try:
        if args.mode=='visuals':screenshots(device,args.agent)
        elif args.mode=='dialogs':dialog_screenshots(device,args.agent)
        elif args.mode=='search-visuals':
            root=device.output
            for theme in ('day','night'):
                visuals['theme'](device,theme)
                results={}
                for renderer in ('compose','rust'):
                    device.output=root/(theme+'-'+renderer);device.output.mkdir(parents=True,exist_ok=True)
                    results[renderer]=search(device,renderer,args.agent)
                assert results['compose']==results['rust'],'Search state/digest mismatch'
                device.output=root;device.save(theme+'-search-comparison.json',{'match':True,'results':results})
                for name in ('gif-search-empty','gif-search-typing'):
                    paths=[str(root/(theme+'-'+r)/(name+'.png')) for r in ('compose','rust')]
                    subprocess.check_call(['swift','scripts/render-benchmark/panel-pair.swift',*paths,str(root/(theme+'-'+name+'-pair.png'))])
        else:
            root=device.output;results={}
            for renderer in ('compose','rust'):
                device.output=root/renderer;device.output.mkdir(parents=True,exist_ok=True)
                results[renderer]=(workflow(device,renderer) if args.mode=='workflow' else search(device,renderer,args.agent))
            assert results['compose']==results['rust'],'Renderer fixture state/digest mismatch'
            device.output=root;device.save('comparison.json',{'match':True,'results':results})
    finally:device.restore_accessibility(original)

if __name__=='__main__':main()
