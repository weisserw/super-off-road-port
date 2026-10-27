"""Original natural body-pose graphics observations from a guarded natural cache.

Source oracle: physical0BDF..0C19 body pose /104C two tile rows. Canonical pose
words0..415 describe the original416 images; pixels are read from original
VRAM and compared with independently addressed local-ROM bytes, never native
render output. This does not require instantaneous RAM selector/NMI alignment.
"""
import argparse,hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,'/root/romatlas/scripts');sys.path.insert(0,str(Path(__file__).resolve().parent))
from native_debug_client import PortClient
from original_inventory import ReferenceSession

def sha(data):return hashlib.sha256(data).hexdigest()
def dump(path,v):path.write_text(json.dumps(v,indent=2)+'\n')
def run(out):
 out.mkdir(parents=True,exist_ok=False)
 origin=ROOT/'modernize/evidence/iteration-003/R2-original-contact/contact.snapshot.json'
 assert sha(origin.read_bytes())=='a5c22e7e46e5a1d72b64664143e34e80adafe1a5db352780c90501fb9f1af4e4'
 assert json.loads(origin.read_text())['artificial'] is False
 rom=(ROOT/'input/game.rom').read_bytes();catalogue={}
 for word in range(416):
  shifted=(word&0xfff8)>>1;selector=((word&7)<<6)|((shifted&255)<<8)|(shifted>>8)
  base=0x10000 if selector&0x8000 else 0x8000
  upper=base+(selector&0x7fff);lower=base+((selector+0x200)&0x7fff)
  digest=sha(rom[upper:upper+64]+rom[lower:lower+64]);catalogue.setdefault(digest,[]).append(word)
 with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(ROOT/'input/game.rom')],seconds=240,max_requests=500,evidence=out/'transport') as client:
  client.request('emulator.start');assert client.request('rom.info')['sha256']==sha(rom)
  result=client.request('emulator.snapshot.import',{'path':str(origin)});assert not result.get('artificial',False)
  session=ReferenceSession(client,native=False);session.advance(1);session.bookmark('first-observation');observations=[];flat=set();tilted=set();captures={}
  def observe():
   data=bytes(client.request('emulator.memory',{'space':'vram','address':0xc000,'length':0x300})['bytes']);images=[]
   for slot in range(4):
    digest=sha(data[slot*64:slot*64+64]+data[0x200+slot*64:0x200+slot*64+64]);words=catalogue.get(digest)
    assert words is not None, f'original truck{slot} graphics outside source pose catalogue'
    if all(w<32 for w in words):flat.update(words)
    if all(w>=32 for w in words):tilted.update(words)
    images.append({'slot':slot,'tile_rows_sha256':digest,'source_pose_candidates':words})
   return {'suffix_frames':sum(s.get('count',0)for s in session.recipe if s['kind']=='advance'),'images':images,'staged_source_words':client.request('emulator.memory',{'space':'wram','address':0x7a,'length':8})['bytes']}
  for index in range(101):
   q=observe();observations.append(q)
   for name,ready in [('flat',bool(flat)),('tilted',bool(tilted))]:
    if ready and name not in captures:
     client.request('emulator.framebuffer',{'path':str(out/(name+'.ppm'))});captures[name]=q
   if len(flat)>=2 and tilted:break
   session.advance(20)
  else:raise AssertionError('bounded natural pose setup did not establish turning and tilt coverage')
  last=observations[-1]
  total=last['suffix_frames']
  session.restore('first-observation')
  if total>1:session.advance(total-1)
  assert observe()==last, 'natural original pose branch did not replay'
  dump(out/'observations.json',observations);dump(out/'recipe.json',{'origin':str(origin.relative_to(ROOT)),'sha256':sha(origin.read_bytes()),'natural_origin_recipe':'modernize/evidence/iteration-003/R2-original-contact/recipe.json','steps':session.recipe,'rom_sha256':sha(rom),'source_sha256':sha((ROOT/'reference/source/game.asm').read_bytes()),'reference_binary_sha256':sha(Path('/root/romatlas/target/release/romatlas').read_bytes())})
  dump(out/'result.json',{'passed':True,'flat_pose_candidates':sorted(flat),'tilted_pose_candidates':sorted(tilted),'observed_frames':len(observations),'persistent_branch_validated':True,'captures':captures,'scope':'fresh original natural flat/turning and tilted truck images match original two-row ROM catalogue; ambiguous identical frames retain all candidates; no native/original whole-frame timing equality claim'})
  print('original pose coverage',sorted(flat),sorted(tilted),flush=True)
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,required=True);a=parser.parse_args();run(a.output.resolve())
