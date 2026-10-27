"""Bounded initial setup probe; retained native/original identities and captures."""
import sys,json
from pathlib import Path
sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
root=Path('/root/super-off-road')
with PortClient([str(root/'port/target/release/super-off-road-port'),'--rom',str(root/'input/game.rom'),'--debug-stdio'],seconds=90,max_requests=70,evidence=root/'debug/artifacts/setup-native-001') as p:
 print('native-api',json.dumps(p.request('api.describe'))[:1200],flush=True)
 for n in [0,32,140,110,100]:
  if n:p.request('port.advance',{'ticks':n,'timeout_ms':30000})
  s=p.request('port.state');Path(p.evidence/f'state-{n}.json').write_text(json.dumps(s,indent=2)); print('native',n,json.dumps(s)[:600],flush=True);p.frame(f'frame-{n}.ppm')
with PortClient(['/root/romatlas/target/release/romatlas','serve','--rom',str(root/'input/game.rom')],seconds=90,max_requests=20,evidence=root/'debug/artifacts/setup-original-001') as p:
 print('original-api',json.dumps(p.request('api.describe'))[:1700],flush=True);print('start',json.dumps(p.request('emulator.start'))[:500],flush=True)
 print('caps',json.dumps(p.request('emulator.capabilities'))[:3000],flush=True)
 for n in [1,30,150,150]:
  result=p.request('emulator.run_fast',{'unit':'frame','count':n,'max_operations':50000000,'timeout_ms':30000,'capture_frame':True}); print('orig',n,json.dumps(result)[:800],flush=True)
  p.request('emulator.framebuffer',{'path':str(p.evidence/f'frame-{p.serial}.ppm')})
