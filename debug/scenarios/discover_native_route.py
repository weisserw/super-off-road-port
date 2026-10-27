import sys,json
from pathlib import Path
sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
root=Path('/root/super-off-road')
with PortClient([str(root/'port/target/release/super-off-road-port'),'--rom',str(root/'input/game.rom'),'--debug-stdio'],seconds=120,max_requests=120,evidence=root/'debug/artifacts/native-route-001') as p:
 def state(label):
  s=p.request('port.state')['game'];Path(p.evidence/f'{label}.json').write_text(json.dumps(s,indent=2));print(label,s['scene'],s['flags'],json.dumps(s['transition']['menu'])[:500],flush=True);return s
 def advance(n):p.request('port.advance',{'ticks':n,'timeout_ms':30000})
 def inp(a):p.request('port.input',{'actions':a})
 advance(360);state('warm');inp(['start']);advance(1);inp([]);advance(100);state('names');inp(['select']);advance(1);inp([]);advance(40);state('joined');
 advance(400);state('timeout');p.frame('after-name.ppm')
