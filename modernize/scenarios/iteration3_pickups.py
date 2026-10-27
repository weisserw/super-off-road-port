"""Natural attract pickup contracts; original discovery and fresh native replay.

Source oracle: objects.trace.md / physical45C7..48DB: rotating collector,
old type count+base,79 remaining after collection and80 total services, rising
paired sprites. Zero seed is explicit comparison policy, never player default.
"""
import argparse,hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,'/root/romatlas/scripts')
sys.path.insert(0,str(Path(__file__).resolve().parent))
from native_debug_client import PortClient
from scenario_client import ScenarioSession
from original_inventory import ReferenceSession

def dump(path,value):path.write_text(json.dumps(value,indent=2)+'\n')
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()

def native(binary,out):
 out.mkdir(parents=True,exist_ok=False)
 with PortClient([str(binary),'--rom',str(ROOT/'input/game.rom'),'--debug-stdio','--rng-seed','0000000000'],seconds=300,max_requests=160,evidence=out/'transport') as client:
  session=ScenarioSession(client,native=True);session.reset()
  session.until(conditions=[{'pointer':'/world/pickup/state','op':'ge','value':3}],ticks=30000,timeout_ms=300000)
  collected=client.request('port.state')['game'];p=collected['world']['pickup']
  assert collected['flags']['fault'] is None and p['lifetime']==79
  assert p['state']==7 and p['counts'][0]==1
  assert sum(collected['world']['pending_awards'])==1
  assert [a['tile_attributes']for a in p['sprites']]==[0x36e0,0x36ce]
  assert collected['progression']['active_humans']==0
  assert not any(e['kind']=='sound_requested' and e['details']=={'command':2,'argument':5}for e in collected['events'])
  session.bookmark('collected');client.frame('collected.ppm')
  print('native collected',collected['simulation_ticks'],p,flush=True)
  session.advance(78);last=client.request('port.state')['game'];assert last['world']['pickup']['lifetime']==1
  assert last['world']['pickup']['state']==p['state']
  session.advance(1);expired=client.request('port.state')['game'];q=expired['world']['pickup']
  assert q['state']==0 and q['lifetime']==0 and all(s['coordinates']==0xf000 for s in q['sprites'])
  client.frame('expired.ppm');dump(out/'landmarks.json',{'collected':collected,'last':last,'expired':expired})
  dump(out/'recipe.json',session.export_recipe());checkpoint=client.request('port.checkpoint.export');dump(out/'checkpoint.json',checkpoint)
  # Validate persistent branch setup and fresh recipe. Identity guards remain intact.
  session.restore('collected');session.advance(79);assert client.request('port.state')['game']==expired
  recipe=json.loads((out/'recipe.json').read_text());session.replay(recipe);assert client.request('port.state')['game']==expired
  dump(out/'result.json',{'passed':True,'binary_sha256':sha(binary),'rom_sha256':sha(ROOT/'input/game.rom'),'identity':client.request('api.describe')['identity'],'scope':'natural attract first pickup collection→79 remaining services→expiry, persistent branch and fresh replay','frame_sha256':checkpoint['frame_sha256'],'audio_sha256':checkpoint['audio_sha256']})
  print('native pickup pass',flush=True)

def original(out, minimum=3):
 out.mkdir(parents=True,exist_ok=False)
 with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(ROOT/'input/game.rom')],seconds=600,max_requests=2500,evidence=out/'transport') as client:
  client.request('emulator.start');assert client.request('rom.info')['sha256']==sha(ROOT/'input/game.rom')
  session=ReferenceSession(client,native=False);session.reset();frames=0
  def read(address,length):return client.request('emulator.memory',{'space':'wram','address':address,'length':length})['bytes']
  def state():
   raw=read(0x6c8,14);word=lambda at:int.from_bytes(bytes(raw[at:at+2]),'little')
   sprites=read(0x390,8)
   return {'world':[word(0),word(2)],'height':word(4),'state':word(6),'lifetime':word(8),'counts':[word(10),word(12)],'sprites':[{'coordinates':int.from_bytes(bytes(sprites[i:i+2]),'little'),'tile_attributes':int.from_bytes(bytes(sprites[i+2:i+4]),'little')}for i in [0,4]],'animation':read(0xb,1)[0],'pending_awards':read(0x633,4),'nitro':[read(0x61b+4*i,1)[0]for i in range(4)]}
  # Search chunks shorter than the80-service award window, then rewind/refine.
  for i in range(1500):
   session.bookmark('before');old_frames=frames;session.advance(20);frames+=20
   p=state()
   if p['state']>=minimum:
    session.restore('before');frames=old_frames;break
   if i%100==0:print('original search',frames,p['state'],flush=True)
  else:raise AssertionError('natural pickup collection not reached in30000frames')
  for _ in range(20):
   before=state();session.advance(1);frames+=1;p=state()
   if p['state']>=minimum and p['lifetime']==79:break
  else:raise AssertionError('collection entry not observed after refinement')
  collected=state()
  slot=collected['animation']&3
  kind=before['state']-1
  assert kind in [0,1]
  assert collected['state']==before['counts'][kind]+(7 if kind==0 else 3)
  assert collected['counts'][kind]==min(before['counts'][kind]+1,4)
  if kind==0:
   assert collected['pending_awards'][slot]==(before['pending_awards'][slot]+1+before['counts'][kind])&255
  else:
   def decimal(value):return (value>>4)*10+(value&15)
   assert decimal(collected['nitro'][slot])==decimal(before['nitro'][slot])+1+before['counts'][kind]
  dump(out/'before-collection.json',before)
  session.bookmark('collected');print('original collected',frames,collected,flush=True)
  client.request('emulator.snapshot.export',{'path':str(out/'collected.snapshot.json')});client.request('emulator.framebuffer',{'path':str(out/'collected.ppm')})
  session.advance(78);last=state();assert last['state']==collected['state'] and last['lifetime']==1,(collected,last)
  session.advance(1);expired=state();assert expired['state']==0 and expired['lifetime']==0
  assert all(s['coordinates']==0xf000 for s in expired['sprites'])
  client.request('emulator.framebuffer',{'path':str(out/'expired.ppm')})
  dump(out/'landmarks.json',{'collected':collected,'last':last,'expired':expired})
  dump(out/'recipe.json',{'version':1,'native':False,'steps':session.recipe,'origin':'deterministic original reset; no state writes','rom_sha256':sha(ROOT/'input/game.rom'),'reference_binary_sha256':sha(Path('/root/romatlas/target/release/romatlas')),'frozen_source_sha256':sha(ROOT/'reference/source/game.asm')})
  session.restore('collected');session.advance(79);assert state()==expired
  dump(out/'result.json',{'passed':True,'frames_at_collection':frames,'scope':'natural original first attract pickup collection/expiry; persistent refinement, no state writes','collected_snapshot_sha256':sha(out/'collected.snapshot.json')})
  print('original pickup pass',flush=True)

if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('mode',choices=['native','original']);parser.add_argument('--binary',type=Path);parser.add_argument('--minimum-award',type=int,default=3);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 if args.mode=='native':native(args.binary.resolve(),args.output.resolve())
 else:original(args.output.resolve(),args.minimum_award)
