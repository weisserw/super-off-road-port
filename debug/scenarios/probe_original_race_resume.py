import sys,json
from pathlib import Path
sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
from capture_helpers import png
root=Path('/root/super-off-road');e=root/'debug/artifacts/original-race-resume-001'
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(root/'input/game.rom')],seconds=120,max_requests=30,evidence=e) as p:
 p.request('emulator.start');p.request('emulator.snapshot.import',{'path':str(root/'debug/artifacts/original-route-001/nextcell.snapshot.json')});frames=1415
 p.request('emulator.input',{'buttons':['b']});r=p.request('emulator.run_fast',{'unit':'frame','count':8,'max_operations':100000000,'timeout_ms':30000,'capture_frame':True});assert r['complete'];frames+=8;p.request('emulator.input',{'buttons':[]})
 for n in [150,150,100]:
  r=p.request('emulator.run_fast',{'unit':'frame','count':n,'max_operations':100000000,'timeout_ms':30000,'capture_frame':True});assert r['complete'];frames+=n
  m=p.request('emulator.memory',{'space':'wram','address':0x4d1,'length':64});print(frames,r['state']['main'],m['bytes'],flush=True);(e/f'frame-{frames}.json').write_text(json.dumps({'state':r,'truck':m},indent=2));path=e/f'frame-{frames}.ppm';p.request('emulator.framebuffer',{'path':str(path)});png(path);p.request('emulator.snapshot.export',{'path':str(e/f'frame-{frames}.snapshot.json')})
