"""Natural no-input loss setup from the guarded original shop cache.

Records bounded500-frame landmarks; inspect prompt capture before reusing one.
This is a discovery tool, not an automatic gameplay equivalence assertion.
"""
from pathlib import Path
import sys,json,hashlib
ROOT=Path('/root/super-off-road');sys.path[:0]=[str(ROOT/'modernize/scenarios'),'/root/romatlas/scripts']
from original_inventory import ReferenceSession
from native_debug_client import PortClient
out=Path(sys.argv[1]).resolve();out.mkdir(exist_ok=False)
snap=ROOT/'debug/artifacts/original-route-001/shop.snapshot.json'
assert hashlib.sha256(snap.read_bytes()).hexdigest()=='f67d6775f5683bdc1c4f74fedaa5ebeb5125ebdc4162a1ccd97458b3fa690799'
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(ROOT/'input/game.rom')],seconds=600,max_requests=250,evidence=out/'transport') as client:
 client.request('emulator.start');client.request('emulator.snapshot.import',{'path':str(snap)})
 s=ReferenceSession(client,native=False)
 def read(a,n):return client.request('emulator.memory',{'space':'wram','address':a,'length':n})['bytes']
 def press(a,n=8):s.input(a);s.advance(n);s.input([]);s.advance(4)
 press(['down'],25);press(['right'],4);press(['b']);s.advance(600)
 for i in range(12):
  s.advance(500)
  print(i,read(6,1),read(0x560,6),read(0x5e8,1),flush=True)
  client.request('emulator.framebuffer',{'path':str(out/f'frame-{i}.ppm')})
  client.request('emulator.snapshot.export',{'path':str(out/f'frame-{i}.snapshot.json')})
  (out/f'recipe-{i}.json').write_text(json.dumps({'origin':str(snap),'origin_sha256':hashlib.sha256(snap.read_bytes()).hexdigest(),'steps':s.recipe},indent=2)+'\n')
