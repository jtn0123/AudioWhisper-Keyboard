"""Compare new chrome semantics against Compose, including actionable ancestor geometry."""
import argparse
import copy
import json
from pathlib import Path
import runpy
import unittest

base=runpy.run_path(str(Path(__file__).with_name('compare-panels.py')))
keyboard=runpy.run_path(str(Path(__file__).with_name('compare-a11y.py')))
SIDES={'Close one-handed mode.','Move keyboard to the left.','Move keyboard to the right.'}
SHEET={'Select subtype','English (United States)','English (United Kingdom)'}


def compare(compose,rust,state):
    names=SHEET if state.startswith('subtypes') else SIDES if state.startswith('onehand') else set()
    if names:
        results=[]
        left=base['targets'](compose);right=base['targets'](rust)
        expected=SHEET if state.startswith('subtypes') else {n for n in SIDES if n in left}
        if not expected:raise ValueError('Expected chrome controls absent from Compose')
        for name in expected:
            a=left.get(name);b=right.get(name);reasons=[]
            if a is None or b is None:reasons.append('missing expected label')
            else:
                if a['enabled']!=b['enabled']:reasons.append('enabled state differs')
                if any(x['id']==16 for x in a['actions']) and not any(x['id']==16 for x in b['actions']):reasons.append('click action missing')
                for axis in (0,1):
                    if abs((a['bounds'][axis]+a['bounds'][axis+2]-b['bounds'][axis]-b['bounds'][axis+2])/2)>3:
                        reasons.append('target center differs by more than 3px');break
            results.append({'target':name,'verdict':'bug' if reasons else 'matches','reason':'; '.join(reasons)})
        # The native sheet adds an explicit scrim dismissal target; all underlying keys are occluded.
        if state.startswith('subtypes'):return {'bug_count':sum(x['verdict']=='bug' for x in results),'targets':results}
    else:results=[]
    # Side labels are checked against their actionable containers above. Other virtual keys
    # retain the established punctuation, state, action and physical-key geometry comparison.
    inputs=copy.deepcopy([compose['nodes'],rust['nodes']])
    if state=='extract':
        for index,node in enumerate(inputs[0]):
            if base['label'](node)=='Done':
                target=base['target'](inputs[0],index)
                node['bounds']=target['bounds'];node['actions']=target['actions']
    for nodes in inputs:
        for node in nodes:
            if base['label'](node) in SIDES:node.update(text=None,description=None)
    diffs=keyboard['compare'](*inputs,state)
    return {'bug_count':sum(x['verdict']=='bug' for x in results+diffs),'targets':results,'keyboard':diffs}


class Tests(unittest.TestCase):
    def sheet(self):
        return {'nodes':[{'parent':-1,'class':'android.widget.Button','text':name,'description':name,'bounds':[0,1800+i*100,1080,1900+i*100],
            'visible':True,'enabled':True,'actions':[{'id':16}]} for i,name in enumerate(sorted(SHEET))]}
    def test_sheet_matches(self):self.assertEqual(0,compare(self.sheet(),self.sheet(),'subtypes')['bug_count'])
    def test_wrong_label_state_action_and_geometry_fail(self):
        for change in ({'text':'wrong','description':'wrong'},{'enabled':False},{'actions':[]},{'bounds':[0,1500,1080,1600]}):
            right=self.sheet();right['nodes'][0].update(change)
            self.assertGreater(compare(self.sheet(),right,'subtypes')['bug_count'],0)


def main():
    parser=argparse.ArgumentParser();parser.add_argument('root');args=parser.parse_args();root=Path(args.root)
    result={}
    for theme in ('day','night'):
        for state in ('subtypes','subtypes-compact','onehand-left','onehand-right','incognito','extract'):
            paths=[root/(theme+'-'+r)/(state+'.json') for r in ('compose','rust')]
            if all(p.exists() for p in paths):result[theme+'-'+state]=compare(*(json.loads(p.read_text()) for p in paths),state)
    (root/'comparison.json').write_text(json.dumps(result,indent=2)+'\n')
    counts={k:v['bug_count'] for k,v in result.items()};print(json.dumps(counts))
    if any(counts.values()):raise SystemExit(1)


if __name__=='__main__':main()
