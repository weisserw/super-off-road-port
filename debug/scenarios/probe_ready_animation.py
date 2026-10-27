import sys,json
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent));sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
from initial_adapter import setup_race,advance,state,frame,dump
from capture_helpers import ppm
root=Path('/root/super-off-road');e=root/'debug/artifacts/ready-animation-001';e.mkdir()
with PortClient([str(root/'port/target/release/super-off-road-port'),'--rom',str(root/'input/game.rom'),'--debug-stdio'],seconds=90,max_requests=120,evidence=e/'native') as p:
 setup_race(p,e,True);advance(p,1);r=p.request('port.run_until',{'ticks':400,'timeout_ms':30000,'conditions':[{'pointer':'/transition/race/phase','value':'CountToReady'},{'pointer':'/transition/race/countdown/state','value':13},{'pointer':'/transition/race/countdown/delay','value':4}]});assert r['matched'];dump(e/'state.json',state(p));frame(p,'countdown')
 a=ppm(root/'debug/artifacts/original-race-resume-001/frame-1723.ppm')[2];b=ppm(e/'native/countdown.ppm')[2];print('mismatch',sum(a[i:i+3]!=b[i:i+3] for i in range(0,len(a),3)),flush=True)
