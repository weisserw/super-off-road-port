"""Independent original score insertion and warm-entry persistence observations.

Guarded natural caches come from the retained original postrace setup/branches.
Reads score storage only; no injected state or fabricated reset recipe.
"""
from pathlib import Path
import json,sys,hashlib
ROOT=Path('/root/super-off-road');sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
sys.path.insert(0,str(ROOT/'modernize/scenarios'))
from original_inventory import ReferenceSession
out=Path(sys.argv[1]).resolve();out.mkdir(exist_ok=False)
origins={'shop':ROOT/'debug/artifacts/original-route-001/shop.snapshot.json','loss':ROOT/'modernize/evidence/iteration-003/R1-original-discovery/frame-9.snapshot.json','warm':ROOT/'modernize/evidence/iteration-003/R1-original-formal/no-frontend.snapshot.json'}
guards={'shop': 'f67d6775f5683bdc1c4f74fedaa5ebeb5125ebdc4162a1ccd97458b3fa690799', 'loss': '7b841dfd0775e0afe4a7ef1e8f3683aab73ba87e3e323e2e634a1740c0670833', 'warm': '2a7c1dcbd34726917e5ecd2972cfdb7e5110a4c9925a9f2da84d30410cfcc0c6'}
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(ROOT/'input/game.rom')],seconds=120,max_requests=40,evidence=out/'transport') as c:
 c.request('emulator.start');states={}
 for name,p in origins.items():
  assert json.loads(p.read_text())['artificial'] is False
  assert hashlib.sha256(p.read_bytes()).hexdigest()==guards[name]
  c.request('emulator.snapshot.import',{'path':str(p)})
  read=lambda a,n:c.request('emulator.memory',{'space':'wram','address':a,'length':n})['bytes']
  states[name]={'score_digits':read(0x578,60),'score_names':read(0x5b4,30),'score_order':read(0x5d2,10),'wallets':read(0x560,24),'origin':str(p.relative_to(ROOT)),'origin_sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
  print(name,states[name]['score_order'],states[name]['score_digits'],flush=True)
 (out/'landmarks.json').write_text(json.dumps(states,indent=2)+'\n')

 assert states['shop']['score_order']==list(range(10))
 assert states['loss']['score_order']==[1,2,3,4,5,6,7,8,0,9]
 assert states['loss']['score_digits'][:6]==[0,0,1,0,0,0]
 assert states['loss']['score_digits'][6:]==states['shop']['score_digits'][6:]
 for key in ['score_digits','score_names','score_order']:assert states['loss'][key]==states['warm'][key],key
 (out/'result.json').write_text(json.dumps({'passed':True,'scope':'Natural source score insertion reuses storage0 at rank8, strict tie leaves highest row intact, warm frontend preserves scores/names/order. Guarded natural caches; no state writes.'},indent=2)+'\n')

 session=ReferenceSession(c,native=False);session.input([]);session.advance(320)
 c.request('emulator.framebuffer',{'path':str(out/'warm-highscores.ppm')})
 (out/'display-recipe.json').write_text(json.dumps({'origin':str(origins['warm'].relative_to(ROOT)),'origin_sha256':guards['warm'],'steps':session.recipe},indent=2)+'\n')
