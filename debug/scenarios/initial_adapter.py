"""Bounded Stage5 checks. Source expectations are declared in the manifests.

Only native gameplay navigation is shared with the separate initial-fidelity
adapter. No expected output is learned from a native capture.
"""
import sys,json,hashlib,struct,wave,math
from pathlib import Path
sys.path.insert(0,'/root/romatlas/scripts')
sys.path.insert(0,str(Path(__file__).resolve().parent))
from native_debug_client import PortClient
from capture_helpers import png,ppm
ROOT=Path('/root/super-off-road')

def dump(path,value):path.write_text(json.dumps(value,indent=2)+'\n')
def identity():
 return {name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in ['input/game.rom','port/target/release/super-off-road-port','debug/scenarios/initial_adapter.py']}
def native(context):
 p=context.stack.enter_context(PortClient([str(ROOT/'port/target/release/super-off-road-port'),'--rom',str(ROOT/'input/game.rom'),'--debug-stdio','--rng-seed','0000000000'],seconds=120,max_requests=250,evidence=context.evidence/'native'))
 context.clients['native']=p;return p

def state(p):
 s=p.request('port.state')['game'];assert s['implementation']=='native' and s['flags']['fault'] is None,s['flags'];return s

def advance(p,n):
 r=p.request('port.advance',{'ticks':n,'timeout_ms':30000});assert r['remaining']==0,r;return state(p)
def until(p,scene,bound=1000):
 r=p.request('port.run_until',{'ticks':bound,'timeout_ms':30000,'conditions':[{'pointer':'/scene','value':scene}]});assert r['matched'],r;return state(p)
def inp(p,actions):p.request('port.input',{'actions':actions})
def press(p,actions,n=8,release=4):inp(p,actions);advance(p,n);inp(p,[]);return advance(p,release)
def frame(p,name):return png(p.frame(name+'.ppm'))
def setup_race(p,output,ready=False):
 # Source Start ignored before cold signature; hold across panel's abort poll.
 advance(p,340);inp(p,['start']);until(p,'Names');inp(p,[]);advance(p,100)
 s=state(p);assert s['transition']['menu']['Names']['phase']=='WaitFrame';dump(output/'names-start.json',s)
 press(p,['select'],1,50);s=press(p,['up']);assert s['transition']['menu']['Names']['editors'][0]['row']==6
 press(p,['b']);until(p,'Shop',800);advance(p,100)
 s=state(p);assert s['transition']['menu']['Shop']['phase']=='WaitFrame';dump(output/'shop-start.json',s);frame(p,'shop')
 press(p,['down'],25);press(p,['right'],4);s=state(p);e=s['transition']['menu']['Shop']['selection']['editors'][0];assert (e['row'],e['column'])==(2,1),e
 # Match source prepared pickup parity after omitted CPU/DMA loading lag.
 # This released tick belongs only to the documented paired-ready setup,
 # not the native gameplay smoke recipe or game rules.
 if ready: advance(p,1)
 press(p,['b']);s=until(p,'Ready' if ready else 'Racing');assert s['progression']['active_humans']==1;assert s['actors'][0]['progress']['lap']==1;dump(output/'race-start.json',s);frame(p,'race-start');return s

def audio_wav(p,path):
 first=p.request('port.audio',{'offset':0,'limit':4096});total=first['frames'];samples=[]
 for offset in range(0,total,4096):
  page=first if offset==0 else p.request('port.audio',{'offset':offset,'limit':4096});samples+=page['samples']
 assert samples and first['dropped_frames']==0,first
 with wave.open(str(path),'wb') as f:f.setnchannels(2);f.setsampwidth(2);f.setframerate(48000);f.writeframes(b''.join(struct.pack('<h',max(-32768,min(32767,round(x*32767)))) for x in samples))
 return {'frames':len(samples)//2,'rms':math.sqrt(sum(x*x for x in samples)/len(samples)),'peak':max(map(abs,samples))}

def run(context,entry):
 if entry['id']!='initial-native-smoke':raise ValueError('wrong native check')
 with context.phase('setup'):
  dump(context.evidence/'identity.json',identity());p=native(context);api=p.request('api.describe');dump(context.evidence/'api.json',api);s=state(p);assert s['scene']=='Splashes' and s['runtime']['nmi_frames']==0
 with context.phase('execution'):
  start=context.check('natural startup/name/shop/ready route',lambda:setup_race(p,context.evidence))
  p.request('port.audio.clear');inp(p,['b']);advance(p,30);mid=state(p);assert mid['actors'][0]['world']!=start['actors'][0]['world']
  s=press(p,['b','left','y'],8,8);assert s['scene']=='Racing';assert s['actors'][0]['heading']!=start['actors'][0]['heading'];context.check('bounded driving and steering',lambda:True)
  dump(context.evidence/'race-driven.json',s);frame(p,'race-driven');sound=audio_wav(p,context.evidence/'native-drive.wav');assert sound['peak']>0 and sound['rms']>0;context.check('active finite native audio',lambda:True)
 return {'start':start['simulation_ticks'],'end':s['simulation_ticks'],'audio':sound,'presentation_review_required':True,'scope':'startup+single short initial-race interaction only'}
