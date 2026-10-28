"""Original continue choices from a guarded natural setup, with suffix recipes.

Inspect endpoint captures for shop/frontend routing. Structured assertions cover
input choice only; this script does not claim automated picture equivalence.
"""
from pathlib import Path
import sys,json,hashlib
ROOT=Path('/root/super-off-road');sys.path[:0]=[str(ROOT/'modernize/scenarios'),'/root/romatlas/scripts']
from original_inventory import ReferenceSession
from native_debug_client import PortClient
out=Path(sys.argv[1]).resolve();out.mkdir(exist_ok=False)
snap=ROOT/'modernize/evidence/iteration-003/R1-original-discovery/frame-9.snapshot.json'
assert json.loads(snap.read_text())['artificial'] is False
assert hashlib.sha256(snap.read_bytes()).hexdigest()=='7b841dfd0775e0afe4a7ef1e8f3683aab73ba87e3e323e2e634a1740c0670833'
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(ROOT/'input/game.rom')],seconds=240,max_requests=120,evidence=out/'transport') as client:
 client.request('emulator.start')
 assert client.request('rom.info')['sha256']=='24b687f95bb9737d223738f13c00aeaa7d697fa9e2fd50597b81d0cfa2160daa'
 client.request('emulator.snapshot.import',{'path':str(snap)})
 s=ReferenceSession(client,native=False);s.bookmark('loss')
 def read(a,n):return client.request('emulator.memory',{'space':'wram','address':a,'length':n})['bytes']
 def capture(name):
  q={'active_humans':read(6,1),'choice':read(0x5e8,1),'wallet0':read(0x560,6)}
  client.request('emulator.framebuffer',{'path':str(out/(name+'.ppm'))})
  client.request('emulator.snapshot.export',{'path':str(out/(name+'.snapshot.json'))})
  (out/(name+'.json')).write_text(json.dumps(q,indent=2)+'\n')
  (out/(name+'-recipe.json')).write_text(json.dumps({'origin':str(snap.relative_to(ROOT)),'origin_sha256':hashlib.sha256(snap.read_bytes()).hexdigest(),'producer_recipe':'modernize/evidence/iteration-003/R1-original-discovery/recipe-9.json','steps':s.recipe},indent=2)+'\n')
  print(name,q,flush=True);return q
 s.advance(1);q=capture('prompt');assert q['choice']==[0] and q['active_humans']==[1]
 s.input(['b']);s.advance(1);s.input([]);s.advance(130);capture('yes-shop')
 s.restore('loss');s.input(['right']);s.advance(2);s.input([]);q=capture('no-selected');assert q['choice']==[1]
 s.input(['b']);s.advance(1);s.input([]);s.advance(200);capture('no-frontend')

 (out/'result.json').write_text(json.dumps({'passed':True,'scope':'Original default/Right input choices and captured natural yes/no routes; endpoint interpretation requires review.json.'},indent=2)+'\n')
