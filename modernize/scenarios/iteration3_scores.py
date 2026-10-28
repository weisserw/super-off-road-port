"""Natural insertion and warm frontend highscore display, with fresh replay.

Reauthors the recorded reset/input/tick plan for each build. No checkpoint
identity bypass, state writes or guessed source storage values.
"""
from pathlib import Path
import json,sys
ROOT=Path('/root/super-off-road');sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
from scenario_client import ScenarioSession
binary=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();out.mkdir(exist_ok=False)
with PortClient([str(binary),'--rom',str(ROOT/'input/game.rom'),'--debug-stdio','--rng-seed','0000000000'],seconds=240,max_requests=150,evidence=out/'transport') as c:
 s=ScenarioSession(c,native=True)
 plan=json.loads((ROOT/'modernize/scenarios/iteration3_scores_recipe.json').read_text())
 for step in plan['steps']:
  if step['kind']=='reset':s.reset()
  elif step['kind']=='input':s.input(step['actions'])
  else:assert step['unit']=='tick';s.advance(step['count'])
 q=c.request('port.state')['game'];assert q['scene']=='Highscores' and q['flags']['fault'] is None
 c.frame('warm-highscores.ppm')
 (out/'state.json').write_text(json.dumps(q,indent=2)+'\n');(out/'checkpoint.json').write_text(json.dumps(c.request('port.checkpoint.export'),indent=2)+'\n');r=s.export_recipe();(out/'recipe.json').write_text(json.dumps(r,indent=2)+'\n')
 s.replay(r);assert c.request('port.state')['game']==q
 (out/'result.json').write_text(json.dumps({'passed':True,'ticks':q['simulation_ticks'],'scope':'Natural completed startup, unspent player wallet, loss, no-continue warm frontend and subsequent highscore drawing, exact fresh replay; scoreboard interpretation requires capture review.'},indent=2)+'\n')
 print(q['scene'],q['simulation_ticks'],flush=True)

if len(sys.argv)>3:
 prior=Path(sys.argv[3]).resolve()
 assert json.loads((prior/'state.json').read_text())==json.loads((out/'state.json').read_text())
 assert (prior/'transport/warm-highscores.ppm').read_bytes()==(out/'transport/warm-highscores.ppm').read_bytes()
 old=json.loads((prior/'checkpoint.json').read_text());new=json.loads((out/'checkpoint.json').read_text())
 for key in old:
  if key!='identity':assert old[key]==new[key],key
 (out/'comparison.json').write_text(json.dumps({'passed':True,'state_identical':True,'frame_identical':True,'checkpoint_fields_identical_except_identity':True,'includes':'Full-route PCM hash; fresh input-plan authoring and guarded replay'},indent=2)+'\n')
