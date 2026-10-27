import sys,json
from pathlib import Path
sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
from capture_helpers import png
root=Path('/root/super-off-road');e=root/'debug/artifacts/original-route-001'
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(root/'input/game.rom')],seconds=120,max_requests=80,evidence=e) as p:
 p.request('emulator.start');p.request('emulator.snapshot.import',{'path':str(root/'debug/artifacts/scroll-reference-002/frame-806.snapshot.json')});frames=806
 def step(n):
  global frames
  r=p.request('emulator.run_fast',{'unit':'frame','count':n,'max_operations':100000000,'timeout_ms':30000,'capture_frame':True});assert r['complete'];frames+=n;return r
 def inp(a):p.request('emulator.input',{'buttons':a})
 def capture(label):
  r=p.request('emulator.state');m=p.request('emulator.memory',{'space':'wram','address':0x540,'length':40});print(label,frames,r['main'],m['bytes'],flush=True);(e/f'{label}.json').write_text(json.dumps({'frames':frames,'state':r,'menu':m},indent=2));path=e/f'{label}.ppm';p.request('emulator.framebuffer',{'path':str(path)});png(path);p.request('emulator.snapshot.export',{'path':str(e/f'{label}.snapshot.json')})
 inp(['start']);step(1);inp([]);step(100);capture('names')
 inp(['select']);step(1);inp([]);step(50);capture('joined')
 inp(['up']);step(8);inp([]);step(4);inp(['b']);step(8);inp([]);step(400);capture('shop')
 inp(['down']);step(25);inp([]);step(4);inp(['right']);step(4);inp([]);step(4);capture('nextcell');inp(['b']);step(8);inp([]);step(300);capture('race')
