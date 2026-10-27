"""Natural first pair-contact and cooldown contracts, without state writes.

Source: contact.trace.md scan/cooldown and committed-vs-proposed rules. This
establishes a natural integration path; exhaustive response cases remain in the
independently source-derived Rust contracts. Whole race frames are not aligned
between original and native loading schedules.
"""
import argparse,hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,'/root/romatlas/scripts');sys.path.insert(0,str(Path(__file__).resolve().parent))
from native_debug_client import PortClient
from scenario_client import ScenarioSession
from original_inventory import ReferenceSession

def dump(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def native(binary,out,baseline=None,original_run=None):
 out.mkdir(parents=True,exist_ok=False)
 with PortClient([str(binary),'--rom',str(ROOT/'input/game.rom'),'--debug-stdio','--rng-seed','0000000000'],seconds=180,max_requests=80,evidence=out/'transport') as client:
  session=ScenarioSession(client,native=True);session.reset()
  session.until(conditions=[{'pointer':'/scene','value':'Racing'},{'pointer':'/actors/0/contact_cooldown','op':'gt','value':0}],ticks=15000,timeout_ms=100000)
  contact=client.request('port.state')['game'];ticks=contact['simulation_ticks']
  assert contact['actors'][0]['contact_cooldown']==6 and contact['flags']['fault'] is None
  # Reconstruct the immediately preceding state naturally, preserving identity.
  session.reset();session.advance(ticks-1);before=client.request('port.state')['game']
  assert before['actors'][0]['contact_cooldown']==0
  session.bookmark('before-contact');session.advance(1);assert client.request('port.state')['game']==contact
  client.frame('contact.ppm');session.advance(1);after=client.request('port.state')['game']
  assert after['actors'][0]['contact_cooldown']==5 and after['flags']['fault'] is None
  checkpoint=client.request('port.checkpoint.export');recipe=session.export_recipe()
  dump(out/'landmarks.json',{'before':before,'contact':contact,'after':after});dump(out/'recipe.json',recipe);dump(out/'checkpoint.json',checkpoint)
  session.restore('before-contact');session.advance(2);assert client.request('port.state')['game']==after
  session.replay(recipe);assert client.request('port.state')['game']==after
  dump(out/'result.json',{'passed':True,'ticks_at_contact':ticks,'binary_sha256':sha(binary),'rom_sha256':sha(ROOT/'input/game.rom'),'scope':'natural first actor0 pair contact, cooldown0→6→5, complete state/frame/audio checkpoints, persistent branch and fresh replay'})
  if baseline:
   load=lambda p:json.loads(p.read_text())
   assert load(baseline/'result.json')['passed']
   assert load(baseline/'landmarks.json')==load(out/'landmarks.json')
   assert sha(baseline/'transport/contact.ppm')==sha(out/'transport/contact.ppm')
   prior=load(baseline/'checkpoint.json')
   for key in prior:
    if key!='identity':assert prior[key]==checkpoint[key],key
   if original_run:
    assert load(original_run/'result.json')['passed']
    original_states=load(original_run/'landmarks.json')
    assert [original_states[name]['cooldowns'][0]for name in ['before','contact','after']]==[0,6,5]
   dump(out/'comparison.json',{'passed':True,'exact_semantic_landmarks':['before','contact','after'],'exact_contact_frame_sha256':sha(out/'transport/contact.ppm'),'exact_route_pcm_sha256':checkpoint['audio_sha256'],'checkpoint_fields_identical_except_identity':True,'original_contract':'natural actor0 cooldown0→6→5; source runtime and native whole race frames are not asserted aligned'})
  print('native natural contact',ticks,flush=True)

def original(out):
 out.mkdir(parents=True,exist_ok=False)
 with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(ROOT/'input/game.rom')],seconds=300,max_requests=800,evidence=out/'transport') as client:
  client.request('emulator.start');assert client.request('rom.info')['sha256']==sha(ROOT/'input/game.rom')
  session=ReferenceSession(client,native=False);session.reset();session.advance(1000);session.advance(500);frames=1500
  def read(address,length):return client.request('emulator.memory',{'space':'wram','address':address,'length':length})['bytes']
  def state():
   raw=read(0x4e9,32);word=lambda i:int.from_bytes(bytes(raw[i:i+2]),'little')
   return {'x':[word(i*2)for i in range(4)],'y':[word(8+i*2)for i in range(4)],'heading':[word(16+i*2)for i in range(4)],'speed':[word(24+i*2)for i in range(4)],'cooldowns':read(0x538,4),'animation':read(0xb,1)[0]}
  assert read(0x538,1)==[0]
  for _ in range(400):
   session.bookmark('chunk');prior=frames;session.advance(10);frames+=10
   if read(0x538,1)[0]>0:
    session.restore('chunk');frames=prior;break
  else:raise AssertionError('first original contact not found within5500frames')
  for _ in range(10):
   before=state();session.advance(1);frames+=1;contact=state()
   if contact['cooldowns'][0]>0:break
  else:raise AssertionError('natural contact refinement failed')
  assert before['cooldowns'][0]==0 and contact['cooldowns'][0]==6
  session.bookmark('contact');client.request('emulator.framebuffer',{'path':str(out/'contact.ppm')});client.request('emulator.snapshot.export',{'path':str(out/'contact.snapshot.json')})
  session.advance(1);after=state();assert after['cooldowns'][0]==5
  dump(out/'landmarks.json',{'before':before,'contact':contact,'after':after});dump(out/'recipe.json',{'origin':'deterministic original reset; no inputs/state writes','steps':session.recipe,'rom_sha256':sha(ROOT/'input/game.rom'),'reference_binary_sha256':sha(Path('/root/romatlas/target/release/romatlas')),'frozen_source_sha256':sha(ROOT/'reference/source/game.asm')})
  session.restore('contact');session.advance(1);assert state()==after
  dump(out/'result.json',{'passed':True,'frames_at_contact':frames,'snapshot_sha256':sha(out/'contact.snapshot.json'),'scope':'natural original first actor0 near-pair scan and cooldown0→6→5; persistent refinement and replay branch, not an exhaustive response-case runtime matrix'})
  print('original natural contact',frames,flush=True)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('mode',choices=['native','original']);p.add_argument('--binary',type=Path);p.add_argument('--baseline',type=Path);p.add_argument('--original',type=Path);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 if a.mode=='native':native(a.binary.resolve(),a.output.resolve(),a.baseline,a.original)
 else:original(a.output.resolve())
