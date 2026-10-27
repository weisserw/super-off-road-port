"""Natural original attract hazard setup with bounded branching/refinement."""
import hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,'/root/romatlas/scripts');sys.path.insert(0,str(Path(__file__).resolve().parent))
from native_debug_client import PortClient
from original_inventory import ReferenceSession
out=ROOT/'modernize/evidence/iteration-002/original-hazard-discovery-002';out.mkdir(exist_ok=False)
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(ROOT/'input/game.rom')],seconds=600,max_requests=2000,evidence=out/'transport') as client:
 client.request('emulator.start');assert client.request('rom.info')['sha256']==hashlib.sha256((ROOT/'input/game.rom').read_bytes()).hexdigest()
 session=ReferenceSession(client,native=False);session.reset();frames=0
 def words(address,count):
  b=client.request('emulator.memory',{'space':'wram','address':address,'length':count*2})['bytes'];return [int.from_bytes(bytes(b[i:i+2]),'little')for i in range(0,len(b),2)]
 def state():
  return {'types':words(0x729,6),'x':words(0x735,6),'animation':words(0x759,6),'timers':words(0x765,6),'uses':words(0x771,6),'track_order':client.request('emulator.memory',{'space':'wram','address':0x1b,'length':2})['bytes']}
 for _ in range(40):
  session.bookmark('before-chunk');prior_frames=frames;session.advance(500);frames+=500
  q=client.request('emulator.memory',{'space':'wram','address':0x1b,'length':2})['bytes']
  if q==[2,2]:
   session.restore('before-chunk');frames=prior_frames;break
 else:raise AssertionError('bounded natural attract route did not reach order2')
 for probe_index in range(1000):
  session.advance(1);frames+=1
  # Read one combined span for types/world/animation/timer/uses.
  raw=client.request('emulator.memory',{'space':'wram','address':0x729,'length':84})['bytes']
  val=lambda offset:int.from_bytes(bytes(raw[offset:offset+2]),'little')
  if probe_index % 100 == 0:print('refine',frames,'types',[val(i*2) for i in range(6)],'x',[val(12+i*2) for i in range(6)],flush=True)
  if val(10)==1 and val(12+10)==2236:break
 else:raise AssertionError('order2 hazard initializer not reached')
 start=state();(out/'initialized.json').write_text(json.dumps({'frames':frames,**start},indent=2));print('original initialized',frames,start,flush=True)
 for _ in range(60):
  session.bookmark('before-response');prior_frames=frames;session.advance(20);frames+=20
  timer=words(0x765+10,1)[0]
  if 0<timer<=43:
   session.restore('before-response');frames=prior_frames;break
 else:raise AssertionError('bounded source hazard contact not reached')
 for _ in range(20):
  session.advance(1);frames+=1
  if words(0x765+10,1)[0]==43:break
 else:raise AssertionError('source response entry timer43 not observed')
 response=state();assert response['types'][5]==1 and response['uses'][5]==0
 (out/'response.json').write_text(json.dumps({'frames':frames,**response},indent=2))
 client.request('emulator.framebuffer',{'path':str(out/'response.ppm')});client.request('emulator.snapshot.export',{'path':str(out/'response.snapshot.json')})
 recipe={'version':1,'native':False,'platform':'snes','steps':session.recipe,'origin':'deterministic original reset; no inputs or state writes','rom_sha256':hashlib.sha256((ROOT/'input/game.rom').read_bytes()).hexdigest(),'reference_binary_sha256':hashlib.sha256(Path('/root/romatlas/target/release/romatlas').read_bytes()).hexdigest(),'frozen_source_sha256':hashlib.sha256((ROOT/'reference/source/game.asm').read_bytes()).hexdigest()}
 (out/'recipe.json').write_text(json.dumps(recipe,indent=2));print('original response',frames,response,flush=True)
