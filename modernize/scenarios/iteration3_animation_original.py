"""Fresh original natural nitro animation, guarded natural shop origin.
Read-only WRAM/OAM observations; no native output supplies expected values.
"""
from pathlib import Path
import sys,json,hashlib
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=['/root/romatlas/scripts',str(Path(__file__).parent)]
from native_debug_client import PortClient
from original_inventory import ReferenceSession
out=Path(sys.argv[1]).resolve();out.mkdir(exist_ok=False)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def dump(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
origin=ROOT/'debug/artifacts/original-route-001/shop.snapshot.json'
assert sha(origin)=='f67d6775f5683bdc1c4f74fedaa5ebeb5125ebdc4162a1ccd97458b3fa690799'
assert json.loads(origin.read_text())['artificial'] is False
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(ROOT/'input/game.rom')],seconds=240,max_requests=1100,evidence=out/'transport') as c:
 c.request('emulator.start');assert c.request('rom.info')['sha256']==sha(ROOT/'input/game.rom')
 c.request('emulator.snapshot.import',{'path':str(origin)});s=ReferenceSession(c,native=False)
 def press(actions,count=8,release=4):s.input(actions);s.advance(count);s.input([]);s.advance(release)
 def read(a,n):return c.request('emulator.memory',{'space':'wram','address':a,'length':n})['bytes']
 def observe():return {'animation':read(0xb,1)[0],'cursors':read(0x519,16),'effect_oam':read(0x350,64)}
 press(['b'],1,12);press(['down'],25);press(['right'],4);press(['b']);s.advance(250)
 s.bookmark('before-nitro');before=observe();assert before['cursors'][0]==0
 s.input(['y','b']);observations=[]
 for frame in range(200):
  s.advance(1);observations.append(observe())
  if frame==159:s.input([])
 active=[q for q in observations if 1<=q['cursors'][0]<=9]
 dump(out/'observations.json',observations)
 assert active,'natural nitro animation never started'
 # First write1 and paced advance can occur within the same physical frame.
 assert set(range(2,10)) <= {q['cursors'][0] for q in observations}
 assert set(range(0x20,0x30,2)) <= {q['effect_oam'][2] for q in active}
 for q in active:
  cursor=q['cursors'][0];tile=q['effect_oam'][2]
  # Source frame counter observed after service selects draw-before-advance.
  drawn=cursor-int(q['animation']&3==0)
  assert 1<=drawn<=8
  assert tile==2*drawn+0x1e,(cursor,tile,q['animation'])
  assert q['effect_oam'][3]==0x2a
 assert observations[-1]['cursors'][0]==0
 assert observations[-1]['effect_oam'][1]==0xf0
 final=observations[-1];recipe=list(s.recipe)
 s.restore('before-nitro');s.input(['y','b'])
 for frame in range(200):
  s.advance(1)
  if frame==159:s.input([])
 replayed=observe();dump(out/'replay-final.json',replayed)
 assert replayed==final
 dump(out/'observations.json',observations)
 dump(out/'recipe.json',{'origin':str(origin.relative_to(ROOT)),'origin_sha256':sha(origin),'steps':recipe,'rom_sha256':sha(ROOT/'input/game.rom'),'source_sha256':sha(ROOT/'reference/source/game.asm'),'reference_binary_sha256':sha(Path('/root/romatlas/target/release/romatlas'))})
 dump(out/'result.json',{'passed':True,'scope':'Natural original nitro slot0 all8 draw frames plus terminator/reset/hide;200 per-frame source cursor/OAM observations and persistent replay.','limits':'Cursor1 can be written and advanced in one physical frame; the first tile proves that draw. Exact staged tile checks use the observed pacing counter. No full source/native frame alignment claim.'})
 print('original nitro animation passed',flush=True)
