"""Natural native nitro visual lifecycle, reset-first recipe and exact comparison.
Source rules: rendering.trace.md $07BB/$0804 and motion.trace.md nitro edge.
"""
from pathlib import Path
import sys,json,hashlib
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
from scenario_client import ScenarioSession
binary=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();out.mkdir(exist_ok=False)
prior=Path(sys.argv[3]).resolve() if len(sys.argv)>3 else None
def dump(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
with PortClient([str(binary),'--rom',str(ROOT/'input/game.rom'),'--debug-stdio','--rng-seed','0000000000'],seconds=180,max_requests=250,evidence=out/'transport') as c:
 s=ScenarioSession(c,native=True)
 plan=json.loads((ROOT/'modernize/evidence/iteration-003/M4-starting-lifecycle-002/recipe.json').read_text())
 assert plan['steps'][0]=={'kind':'reset'}
 for step in plan['steps']:
  if step['kind']=='input' and step['actions']==['y','b']:break
  if step['kind']=='reset':s.reset()
  elif step['kind']=='input':s.input(step['actions'])
  else:
   assert step['kind']=='advance' and step['unit']=='tick';s.advance(step['count'])
 else:raise AssertionError('nitro input edge missing from natural recipe')
 before=c.request('port.state')['game'];assert before['scene']=='Racing' and before['actors'][0]['nitro']==1
 s.bookmark('before-nitro');s.input(['y','b']);observations=[];captures={}
 for frame in range(48):
  s.advance(1);q=c.request('port.state')['game'];assert q['flags']['fault'] is None
  observations.append(q)
  if q['actors'][0]['nitro_cursor'] and 'active' not in captures:
   c.frame('active.ppm');captures['active']=frame
  if frame==7:s.input([])
 entries={q['actors'][0]['nitro_cursor'] for q in observations}
 assert set(range(2,10))<=entries,entries
 assert observations[-1]['actors'][0]['nitro_cursor']==0
 assert observations[-1]['actors'][0]['nitro']==0
 c.frame('finished.ppm');checkpoint=c.request('port.checkpoint.export');recipe=s.export_recipe()
 s.restore('before-nitro');s.input(['y','b']);s.advance(8);s.input([]);s.advance(40)
 assert c.request('port.state')['game']==observations[-1]
 s.replay(recipe);assert c.request('port.state')['game']==observations[-1]
 dump(out/'observations.json',observations);dump(out/'checkpoint.json',checkpoint);dump(out/'recipe.json',recipe)
 dump(out/'result.json',{'passed':True,'entries':sorted(entries),'captures':captures,'scope':'Natural native48-frame nitro start/progression/finish, source phase2..9 plus reset, stock consumption, persistent branch and guarded reset-first recipe replay','binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest()})
 if prior:
  assert json.loads((prior/'result.json').read_text())['passed']
  assert json.loads((prior/'observations.json').read_text())==observations
  old=json.loads((prior/'checkpoint.json').read_text())
  for key in old:
   if key!='identity':assert old[key]==checkpoint[key],key
  for name in ['active','finished']:assert (prior/'transport'/(name+'.ppm')).read_bytes()==(out/'transport'/(name+'.ppm')).read_bytes()
  dump(out/'comparison.json',{'passed':True,'all48_full_states_identical':True,'both_complete_frames_identical':True,'checkpoint_fields_including_PCM_identical_except_identity':True})
 print('native nitro lifecycle passed',flush=True)
