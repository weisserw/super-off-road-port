"""Natural postrace/continue branches; original rules in results.trace.md.

Uses the retained reset/input/tick plan, freshly authored for each build identity.
Captures/checkpoints remain local; no memory writes or cross-build imports.
"""
from pathlib import Path
import sys,json
ROOT=Path('/root/super-off-road');sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
from scenario_client import ScenarioSession
binary=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();out.mkdir(exist_ok=False)
with PortClient([str(binary),'--rom',str(ROOT/'input/game.rom'),'--debug-stdio','--rng-seed','0000000000'],seconds=240,max_requests=250,evidence=out/'transport') as client:
 s=ScenarioSession(client,native=True)
 # Reauthor the retained natural input/tick plan against this build. Never import
 # an old checkpoint or edit its identity to make it load on a changed build.
 prefix=ROOT/'modernize/evidence/iteration-003/R1-native-discovery/loss-recipe.json'
 plan=json.loads(prefix.read_text())
 assert plan['native'] and plan['steps'][0]=={'kind':'reset'}
 for step in plan['steps']:
  if step['kind']=='reset':s.reset()
  elif step['kind']=='input':s.input(step['actions'])
  else:
   assert step['kind']=='advance' and step['unit']=='tick'
   s.advance(step['count'])
 assert client.request('port.state')['game']['scene']=='LossPrompt'
 s.bookmark('loss')
 def capture(name):
  q=client.request('port.state')['game'];assert q['flags']['fault'] is None
  (out/(name+'.json')).write_text(json.dumps(q,indent=2)+'\n');client.frame(name+'.ppm')
  (out/(name+'-recipe.json')).write_text(json.dumps(s.export_recipe(),indent=2)+'\n')
  (out/(name+'-checkpoint.json')).write_text(json.dumps(client.request('port.checkpoint.export'),indent=2)+'\n')
  print(name,q['scene'],q['simulation_ticks'],q['progression']['active_humans'],flush=True)
  return q
 capture('prompt')
 s.input(['b']);s.advance(1);s.input([]);q=capture('yes-close');assert q['scene']=='LossClose'
 s.until(conditions=[{'pointer':'/scene','value':'Shop'}],ticks=1000,timeout_ms=60000);q=capture('yes-shop');assert q['progression']['active_humans']==1
 s.advance(130);q=capture('yes-shop-visible');assert q['scene']=='Shop'
 s.restore('loss');s.input(['right']);s.advance(1);s.input([]);capture('no-selected')
 s.input(['b']);s.advance(1);s.input([]);q=capture('no-close');assert q['scene']=='LossClose'
 s.advance(200);q=capture('no-frontend');assert q['scene']=='AttractPanel' and q['progression']['active_humans']==1 # enter_frontend resets default mask
 (out/'result.json').write_text(json.dumps({'passed':True,'scope':'Natural human loss, persistent yes/no branches; source B/Right input and shop/frontend routing assertions.'},indent=2)+'\n')

if len(sys.argv)>3:
 import hashlib
 prior=Path(sys.argv[3]).resolve()
 names=['prompt','yes-close','yes-shop','yes-shop-visible','no-selected','no-close','no-frontend']
 for name in names:
  assert json.loads((prior/(name+'.json')).read_text())==json.loads((out/(name+'.json')).read_text()),name
  assert (prior/'transport'/(name+'.ppm')).read_bytes()==(out/'transport'/(name+'.ppm')).read_bytes(),name
  old=json.loads((prior/(name+'-checkpoint.json')).read_text());new=json.loads((out/(name+'-checkpoint.json')).read_text())
  assert old.keys()==new.keys()
  for key in old:
   if key!='identity':assert old[key]==new[key],(name,key)
 (out/'comparison.json').write_text(json.dumps({'passed':True,'landmarks':names,'frames_identical':True,'checkpoint_fields_identical_except_identity':True,'includes':'PCM hashes and semantic state; same natural input plan freshly authored per build'},indent=2)+'\n')
