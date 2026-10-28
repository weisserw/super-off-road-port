"""Original initial eight-voice music comparison at established source phases."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=['/root/romatlas/scripts',str(ROOT/'debug/scenarios')]
from native_debug_client import PortClient
from scenario_client import ScenarioSession
from initial_sanity_adapter import compare_audio_state
binary=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();original=Path(sys.argv[3]).resolve();out.mkdir(exist_ok=False)
expected=json.loads((original/'music.json').read_text())
with PortClient([str(binary),'--rom',str(ROOT/'input/game.rom'),'--debug-stdio','--rng-seed','0000000000'],seconds=120,max_requests=40,evidence=out/'transport') as client:
 session=ScenarioSession(client,native=True);session.reset();session.advance(83)
 before=client.request('port.state')['game'];assert before['runtime']['nmi_frames']==83
 compare_audio_state(expected['before'],before)
 session.advance(30);after=client.request('port.state')['game'];compare_audio_state(expected['after'],after)
 checkpoint=client.request('port.checkpoint.export')
 (out/'landmarks.json').write_text(json.dumps({'before':before,'after':after},indent=2)+'\n');(out/'checkpoint.json').write_text(json.dumps(checkpoint,indent=2)+'\n');(out/'recipe.json').write_text(json.dumps(session.export_recipe(),indent=2)+'\n')
 (out/'result.json').write_text(json.dumps({'passed':True,'scope':'Eight original/native sequence/duration/pitch/instrument/amplitude vectors at NMI85/native83 and30frames later; source command-handoff lag retained','limits':'No exact original PCM claim; native linear DSP interpolation policy unchanged.'},indent=2)+'\n')
 if len(sys.argv)>4:
  prior=Path(sys.argv[4]);assert json.loads((prior/'landmarks.json').read_text())=={'before':before,'after':after}
  old=json.loads((prior/'checkpoint.json').read_text())
  for key in old:
   if key!='identity':assert old[key]==checkpoint[key],key
  (out/'comparison.json').write_text(json.dumps({'passed':True,'full_semantic_landmarks_identical':True,'all_checkpoint_fields_except_identity_including_PCM_identical':True},indent=2)+'\n')
 print('Original8-voice music phases match.',flush=True)
