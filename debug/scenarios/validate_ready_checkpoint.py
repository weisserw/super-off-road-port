"""Current-build paired-ready setup replay validation; no extra gameplay scope."""
import sys,json,contextlib
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
from initial_adapter import native,setup_race,state,advance,dump,identity
ROOT=Path('/root/super-off-road')
class Context:
 def __init__(self):
  self.evidence=ROOT/'debug/artifacts/ready-checkpoint-current';self.evidence.mkdir(exist_ok=False);self.stack=contextlib.ExitStack();self.clients={}
c=Context()
with c.stack:
 p=native(c);setup_race(p,c.evidence,ready=True);advance(p,1)
 q=p.request('port.run_until',{'ticks':400,'timeout_ms':30000,'conditions':[{'pointer':'/transition/race/phase','value':'CountToReady'},{'pointer':'/transition/race/countdown/state','value':13},{'pointer':'/transition/race/countdown/delay','value':4}]});assert q['matched']
 before=state(p);assert before['world']['pickup']['state']==2
 checkpoint=p.request('port.checkpoint.export');dump(c.evidence/'checkpoint.json',checkpoint);dump(c.evidence/'identity.json',identity())
 restored=p.request('port.checkpoint.import',{'checkpoint':checkpoint,'timeout_ms':30000});assert state(p)==before
 after=p.request('port.checkpoint.export');assert after==checkpoint
 dump(c.evidence/'validation.json',{'status':'passed','setup':'source Ready CountToReady13/4, track4, pickup2; same natural paired recipe','semantic_exact':True,'checkpoint_exact':True,'frame_sha256':checkpoint['frame_sha256'],'audio_sha256':checkpoint['audio_sha256'],'identity':checkpoint['identity'],'import':restored})
print('fresh paired-ready checkpoint import/export semantic/frame/audio validation PASS')
