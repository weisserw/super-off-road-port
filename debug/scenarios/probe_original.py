import sys,json
from pathlib import Path
sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
root=Path('/root/super-off-road')
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(root/'input/game.rom')],seconds=120,max_requests=25,evidence=root/'debug/artifacts/setup-original-002') as p:
 print('api',json.dumps(p.request('api.describe'))[:600],flush=True);print('start',json.dumps(p.request('emulator.start'))[:300],flush=True)
 print('caps',json.dumps(p.request('emulator.capabilities'))[:3500],flush=True)
 for n in [1,30,150,150]:
  result=p.request('emulator.run_fast',{'unit':'frame','count':n,'max_operations':50000000,'timeout_ms':30000,'capture_frame':True}); print('orig',n,json.dumps(result)[:1000],flush=True)
  p.request('emulator.framebuffer',{'path':str(p.evidence/f'frame-{p.serial}.ppm')})
