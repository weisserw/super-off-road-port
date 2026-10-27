"""Source-backed iteration2 route and dynamic-hazard integration scenarios."""
import hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,'/root/romatlas/scripts');sys.path.insert(0,str(ROOT/'debug/scenarios'));sys.path.insert(0,str(Path(__file__).resolve().parent))
from native_debug_client import PortClient
from scenario_client import ScenarioSession
from original_inventory import ReferenceSession
from capture_helpers import png

def dump(path,value):path.write_text(json.dumps(value,indent=2)+'\n')
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()

def native_run(context,entry):
 binary=ROOT/entry['binary'];out=context.evidence
 with context.phase('setup'):
  client=context.stack.enter_context(PortClient([str(binary),'--rom',str(ROOT/'input/game.rom'),'--debug-stdio','--rng-seed','0000000000'],seconds=300,max_requests=180,evidence=out/'native'));context.clients['native']=client
  session=ScenarioSession(client,native=True);session.reset()
  session.until(conditions=[{'pointer':'/scene','value':'Racing'},{'pointer':'/progression/source_order','value':2}],ticks=30000,timeout_ms=300000)
  state=client.request('port.state')['game'];assert state['flags']['fault'] is None and state['progression']['original_track']==2
  assert [d['kind'] for d in state['world']['dynamic']]==[0,1,1,1,1,1]
  setup_recipe=session.export_recipe();dump(out/'setup-recipe.json',setup_recipe)
  dump(out/'setup-checkpoint.json',client.request('port.checkpoint.export'))
 with context.phase('execution'):
  session.until(conditions=[{'pointer':'/world/dynamic/5/timer','value':43}],ticks=6000,timeout_ms=300000)
  response=client.request('port.state')['game'];object=response['world']['dynamic'][5]
  def response_check():
   assert response['progression']['source_order']==2 and response['progression']['active_humans']==0
   assert (object['kind'],object['animation'],object['uses'])==(1,34,0)
   assert object['placement']=={'world':[2236,6980],'height':84}
   assert object['sprite']=={'coordinates':0x4731,'tile_attributes':0x356c}
  context.check('natural original-layout response entry and projection',response_check)
  png(client.frame('response.ppm'));session.bookmark('response')
  session.advance(44)
  depleted=client.request('port.state')['game'];object=depleted['world']['dynamic'][5]
  def depleted_check():
   assert (object['kind'],object['timer'],object['uses'],object['animation'])==(2,0,65535,0)
   assert object['sprite']['tile_attributes']==0x256e
   assert not any(e['kind']=='sound_requested' and e['details']=={'command':2,'argument':7} for e in depleted['events']), 'attract hazard contact is silent'
  context.check('zero encoded uses permits one complete response then depletion',depleted_check)
  png(client.frame('depleted.ppm'));depletion_state=depleted
  # A fresh bounded continuation witnesses lap progression through the same
  # actual immutable route/AI owners used by the hazard scenario.
  for _ in range(80):
   if max(a['progress']['lap'] for a in depleted['actors'])>=2:break
   session.advance(25);depleted=client.request('port.state')['game']
   assert depleted['flags']['fault'] is None and depleted['progression']['source_order']==2
  else:raise AssertionError('bounded track2 lap transition not reached')
  def lap_check():assert max(a['progress']['lap'] for a in depleted['actors'])>=2
  context.check('natural AI waypoint traversal reaches a second lap',lap_check)
  dump(out/'lap-recipe.json',session.export_recipe())
  # Keep the actual depletion state separately from the later lap observation.
  session.restore('response');session.advance(44);assert client.request('port.state')['game']==depletion_state
  dump(out/'landmarks.json',{'response':response,'depleted':depletion_state,'lap':depleted})
  recipe=session.export_recipe();dump(out/'response-recipe.json',recipe)
  checkpoint=client.request('port.checkpoint.export');dump(out/'response-checkpoint.json',checkpoint)
  session.replay(recipe);assert client.request('port.state')['game']==depletion_state
  dump(out/'identity.json',{'binary_sha256':digest(binary),'api':client.request('api.describe')})
 return {'scope':'natural attract track6 race completion→track2 response/depletion and AI lap transition; guarded fresh replay','frame_sha256':checkpoint['frame_sha256'],'audio_sha256':checkpoint['audio_sha256'],'presentation_review_required':True}

def original_run(context,entry):
 out=context.evidence;snapshot=ROOT/entry['snapshot']
 with context.phase('setup'):
  assert digest(snapshot)==entry['snapshot_sha256']
  client=context.stack.enter_context(PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(ROOT/'input/game.rom')],seconds=180,max_requests=200,evidence=out/'original'));context.clients['original']=client
  client.request('emulator.start');assert client.request('rom.info')['sha256']=='24b687f95bb9737d223738f13c00aeaa7d697fa9e2fd50597b81d0cfa2160daa'
  result=client.request('emulator.snapshot.import',{'path':str(snapshot)});assert not result.get('artificial',False)
  session=ReferenceSession(client,native=False)
  def words(address,count):
   b=client.request('emulator.memory',{'space':'wram','address':address,'length':count*2})['bytes'];return [int.from_bytes(bytes(b[i:i+2]),'little')for i in range(0,len(b),2)]
  def observe():
   return {'kind':words(0x729+10,1)[0],'animation':words(0x759+10,1)[0],'timer':words(0x765+10,1)[0],'uses':words(0x771+10,1)[0],'sprite':words(0x200+0x198+5*4,2),'laps':client.request('emulator.memory',{'space':'wram','address':0x4e5,'length':4})['bytes']}
 with context.phase('execution'):
  response=observe()
  def response_check():assert (response['kind'],response['animation'],response['timer'],response['uses'],response['sprite'])==(1,34,43,0,[0x4731,0x356c])
  context.check('original natural hazard response and projected sprite',response_check)
  session.advance(44);depleted=observe()
  def depleted_check():assert (depleted['kind'],depleted['timer'],depleted['uses'],depleted['animation'],depleted['sprite'][1])==(2,0,65535,0,0x256e)
  context.check('original hazard completes44 remaining services and depletes',depleted_check)
  client.request('emulator.framebuffer',{'path':str(out/'depleted.ppm')});png(out/'depleted.ppm')
  at_depletion=depleted.copy()
  # Witness a naturally completed lap, without synchronizing source physical
  # frames to native loading latency or assuming the same leading truck.
  for _ in range(80):
   if max(depleted['laps'])>=2:break
   session.advance(25);depleted={'laps':client.request('emulator.memory',{'space':'wram','address':0x4e5,'length':4})['bytes']}
  else:raise AssertionError('original second lap bound exhausted')
  def lap_check():assert max(depleted['laps'])>=2
  context.check('original natural route reaches a second lap',lap_check)
  dump(out/'observations.json',{'response':response,'depleted':at_depletion,'later':depleted,'suffix_recipe':session.recipe,'origin_snapshot':entry['snapshot'],'origin_sha256':entry['snapshot_sha256'],'natural_origin_recipe':'modernize/evidence/iteration-002/original-hazard-discovery-002/recipe.json'})
 return {'scope':'original natural attract hazard and lap response; explicit cached natural origin, no state writes','presentation_review_required':True}

def run(context,entry):
 return original_run(context,entry) if entry['id']=='original-hazard-route' else native_run(context,entry)
