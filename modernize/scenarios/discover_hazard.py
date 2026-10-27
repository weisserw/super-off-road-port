"""Bounded natural attract setup for a dynamic response; no state writes."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
from scenario_client import ScenarioSession
out=ROOT/'modernize/evidence/iteration-002/native-hazard-discovery-001'
out.mkdir(exist_ok=False)
with PortClient([str(ROOT/'modernize/evidence/iteration-002/starting-port'),'--rom',str(ROOT/'input/game.rom'),'--debug-stdio','--rng-seed','0000000000'],seconds=300,max_requests=100,evidence=out/'transport') as client:
 session=ScenarioSession(client,native=True);session.reset()
 result=session.until(conditions=[{'pointer':'/scene','value':'Racing'},{'pointer':'/progression/source_order','value':2}],ticks=30000,timeout_ms=300000)
 state=client.request('port.state')['game'];assert state['flags']['fault'] is None
 print('track2 reached',state['simulation_ticks'],[d['kind'] for d in state['world']['dynamic']],flush=True)
 session.bookmark('race-start')
 (out/'race-start.json').write_text(json.dumps(state,indent=2))
 (out/'race-start-checkpoint.json').write_text(json.dumps(client.request('port.checkpoint.export'),indent=2))
 # Slot5 is the first nonzero placement for original order2; choose its response
 # specifically rather than polling unbounded actors or modifying a placement.
 result=session.until(conditions=[{'pointer':'/world/dynamic/5/timer','op':'gt','value':0},{'pointer':'/world/dynamic/5/timer','op':'le','value':43}],ticks=6000,timeout_ms=300000)
 state=client.request('port.state')['game'];assert state['progression']['source_order']==2
 (out/'response.json').write_text(json.dumps(state,indent=2));client.frame('response.ppm')
 (out/'response-checkpoint.json').write_text(json.dumps(client.request('port.checkpoint.export'),indent=2))
 (out/'recipe.json').write_text(json.dumps(session.export_recipe(),indent=2))
 print('response',state['simulation_ticks'],state['world']['dynamic'][5],flush=True)
