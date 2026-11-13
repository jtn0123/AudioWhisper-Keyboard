"""Clipboard/GIF semantic comparison, accounting for icon-only Compose controls and modal occlusion."""
import argparse
import copy
import json
from pathlib import Path
import re
import runpy
import unittest

base=runpy.run_path(str(Path(__file__).with_name('compare-panels.py')))
keys=runpy.run_path(str(Path(__file__).with_name('compare-a11y.py')))
HEADERS={'Back','Enable clipboard history','Disable clipboard history','Clear history','Filter history','Clipboard Delete key'}

def label(node):return node.get('description') or node.get('text')

def normalize(data):
    data=copy.deepcopy(data);nodes=data['nodes']
    modal=any(label(n) in ('Pin item','Unpin item') or (label(n) or '').startswith('Are you sure') for n in nodes)
    for index,node in enumerate(nodes):
        parent=node['parent']
        while parent>=0:
            if not nodes[parent].get('enabled',True):node['enabled']=False
            parent=nodes[parent]['parent']
        name=label(node) or ''
        if name=='Delete' and node['bounds'][1]<1700:
            node['text']='Clipboard Delete key';node['description']='Clipboard Delete key';name='Clipboard Delete key'
        if re.match(r'^[A-Z][a-z]{2} \d{1,2}, \d{4},',name):
            node['text']='Fixture timestamp';node['description']='Fixture timestamp'
        if modal and name.startswith('aw-test-'):
            # Grid targets are behind the modal scrim. The preview repeats one item's
            # text; screenshot review checks it, while the menu's named actions are compared here.
            node['visible']=False
            base['target'](nodes,index)['visible']=False
    return data


def compare(compose,rust):
    compose,rust=normalize(compose),normalize(rust)
    keyboard_differences=[]
    if any(label(n)=='Back to GIFs' for n in rust['nodes']):
        boundary=min(n['bounds'][1] for n in rust['nodes'] if n.get('textEntryKey'))
        keyboard=copy.deepcopy([compose,rust])
        for data in keyboard:
            for node in data['nodes']:
                if node['bounds'][1]<boundary:
                    node.update(text=None,description=None,textEntryKey=False)
        keyboard_differences=keys['compare'](keyboard[0]['nodes'],keyboard[1]['nodes'],'gif-search')
        for data in (compose,rust):
            for node in data['nodes']:
                if node['bounds'][1]>=boundary:node['visible']=False
    # Compose's header icons have no labels. Match those public button rectangles to
    # native names, then apply the same enabled/click/geometry checks to each one.
    expected={label(n) for n in compose['nodes']}
    for node in rust['nodes']:
        name=label(node)
        if name not in HEADERS or name in expected or node['bounds'][1]>1700:continue
        b=node['bounds'];cx=(b[0]+b[2])/2;cy=(b[1]+b[3])/2
        candidates=[n for n in compose['nodes'] if not label(n) and (n.get('clickable') or n.get('class')=='android.widget.Button') and abs((n['bounds'][1]+n['bounds'][3])/2-cy)<20 and n['bounds'][2]-n['bounds'][0]<150]
        if candidates:
            target=min(candidates,key=lambda n:abs((n['bounds'][0]+n['bounds'][2])/2-cx))
            if abs((target['bounds'][0]+target['bounds'][2])/2-cx)>20:continue
            target['text']=name;target['description']=name
    result=base['compare'](compose,rust)
    for row in result['targets']:
        if row['target']=='Clipboard Delete key' and row['reason']=='unexplained extra label':row.update(verdict='acceptable',reason='explicit native label for Compose pointer-only delete key')
    result['keyboard']=keyboard_differences
    result['bug_count']=sum(r['verdict']=='bug' for r in result['targets']+keyboard_differences)
    return result


class Tests(unittest.TestCase):
    def fixture(self):
        return {'nodes':[{'parent':-1,'class':'android.widget.Button','text':'Paste','description':'Paste','bounds':[600,1800,1000,1920],'visible':True,'enabled':True,'actions':[{'id':16}]}]}
    def test_matches(self):self.assertEqual(0,compare(self.fixture(),self.fixture())['bug_count'])
    def test_rejects_missing_action_wrong_state_and_displaced_menu(self):
        for change in ({'actions':[]},{'enabled':False},{'bounds':[600,1900,1000,2020]},{'text':'Wrong','description':'Wrong'}):
            right=self.fixture();right['nodes'][0].update(change)
            self.assertGreater(compare(self.fixture(),right)['bug_count'],0)
    def test_modal_controls_are_still_strict(self):
        for change in ({'actions':[]},{'enabled':False},{'bounds':[600,1900,1000,2020]},{'text':'Wrong','description':'Wrong'}):
            left=self.fixture();left['nodes'][0].update(text='No',description='No')
            left['nodes'].append(dict(left['nodes'][0],text='Are you sure?',description='Are you sure?',bounds=[100,1700,500,1800]))
            right=copy.deepcopy(left);right['nodes'][0].update(change)
            self.assertGreater(compare(left,right)['bug_count'],0)
    def test_occluded_compose_leaf_hides_its_click_wrapper(self):
        data=self.fixture();data['nodes'][0].update(text=None,description=None)
        data['nodes'].append(dict(data['nodes'][0],parent=0,text='aw-test-1',description=None,actions=[],**{'class':'android.widget.TextView'}))
        data['nodes'].append(dict(data['nodes'][0],text='Are you sure?',description='Are you sure?'))
        self.assertNotIn('aw-test-1',base['targets'](normalize(data)))
    def test_disabled_ancestor_is_effective(self):
        data=self.fixture();data['nodes'].insert(0,dict(data['nodes'][0],text=None,description=None,enabled=False));data['nodes'][1]['parent']=0
        self.assertFalse(normalize(data)['nodes'][1]['enabled'])


def main():
    parser=argparse.ArgumentParser();parser.add_argument('root');args=parser.parse_args();root=Path(args.root)
    results={}
    for theme in ('day','night'):
        for state in ('clipboard-empty','clipboard-items','clipboard-menu','clipboard-clear-dialog','gif-no-key','gif-search-empty','gif-search-typing'):
            paths=[root/(theme+'-'+r)/(state+'.json') for r in ('compose','rust')]
            if not all(p.exists() for p in paths):continue
            results[theme+'-'+state]=compare(*(json.loads(p.read_text()) for p in paths))
    (root/'comparison.json').write_text(json.dumps(results,indent=2)+'\n')
    print(json.dumps({k:v['bug_count'] for k,v in results.items()}))
    if any(v['bug_count'] for v in results.values()):raise SystemExit(1)

if __name__=='__main__':main()
