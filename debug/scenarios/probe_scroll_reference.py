import sys,json
from pathlib import Path
sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
root=Path('/root/super-off-road');e=root/'debug/artifacts/scroll-reference-001'
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(root/'input/game.rom')],seconds=120,max_requests=80,evidence=e) as p:
 p.request('emulator.start');frames=0
 for n in [181,150,50,25,25,25,25,25,25,25]:
  r=p.request('emulator.run_fast',{'unit':'frame','count':n,'max_operations':50000000,'timeout_ms':30000,'capture_frame':True});frames+=n
  m=p.request('emulator.memory',{'space':'wram','address':0x5e2,'length':2});print(frames,m,r['state']['main'],flush=True)
  p.request('emulator.framebuffer',{'path':str(e/f'frame-{frames}.ppm')})
