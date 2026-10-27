"""Initial selected original-fidelity comparisons, kept separate from gameplay."""
import sys,json,hashlib,wave,math,struct
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent));sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
from initial_adapter import native,state,advance,until,frame,dump,identity,audio_wav,setup_race
from capture_helpers import ppm,png
ROOT=Path('/root/super-off-road')

def exact(original,native):
 ow,oh,op=ppm(original);nw,nh,np=ppm(native);assert (ow,oh)==(nw,nh)
 mismatch=sum(op[i:i+3]!=np[i:i+3] for i in range(0,len(op),3))
 assert mismatch==0,f'{original.name} vs {native.name}: {mismatch}/{ow*oh} pixel mismatches'
 return {'pixels':ow*oh,'mismatches':mismatch,'sha256':hashlib.sha256(op).hexdigest()}

def original_audio_state(r):
 out={}
 for name,address in [('sequence',0x30),('duration',0x80),('dsp_pitch',0x10),('instrument_gain',0x20)]:
  out[name]=r.request('emulator.memory',{'space':'aram','address':address,'length':16})['bytes']
 return {'sequence':[int.from_bytes(bytes(out['sequence'][i:i+2]),'little') for i in range(0,16,2)],'duration':out['duration'][::2],'pitch':[int.from_bytes(bytes(out['dsp_pitch'][i:i+2]),'little') for i in range(0,16,2)],'sample':out['instrument_gain'][::2],'amplitude':out['instrument_gain'][1::2]}

def compare_audio_state(original,native):
 voices=native['audio']['voices']
 for name,expected in original.items():
  actual=[voice[name] for voice in voices];assert actual==expected,(name,expected,actual)
 return {'voices':8,'fields':['sequence','duration','pitch','sample','amplitude'],'exact':True}

def run(context,entry):
 with context.phase('setup'):
  dump(context.evidence/'identity.json',identity());n=native(context)
  r=context.stack.enter_context(PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(ROOT/'input/game.rom')],seconds=180,max_requests=60,evidence=context.evidence/'original'));context.clients['original']=r
  api=r.request('api.describe');assert api['platform']=='snes';r.request('emulator.start');assert r.request('rom.info')['sha256']==identity()['input/game.rom']
 with context.phase('execution'):
  observations={'frames':[]}
  last=0
  reuse=entry.get('reuse_frontend')
  if reuse:
   base=ROOT/reuse;prior=json.load(open(base/'identity.json'))
   if prior['port/target/release/super-off-road-port']!=identity()['port/target/release/super-off-road-port']:
    assert entry.get('reuse_build_change_reason')
    for path,expected in entry['reuse_producer_sha256'].items():assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==expected
    dump(context.evidence/'frontend-impact-review.json',{'reason':entry['reuse_build_change_reason'],'producer_sha256':entry['reuse_producer_sha256'],'prior_identity':prior})
   for label in ['license','software','scroll-0132','scroll-0142']:
    value=context.check(label+' retained exact source pair',lambda label=label:exact(base/'original'/f'{label}.ppm',base/'native'/f'{label}.ppm'));observations['frames'].append({'label':label,'reused_from':str(base),**value})
  for label,original_frame,native_tick in ([] if reuse else [('license',181,172),('software',331,282)]):
   q=r.request('emulator.run_fast',{'unit':'frame','count':original_frame-last,'max_operations':100000000,'timeout_ms':30000,'capture_frame':True});assert q['complete'];last=original_frame
   original=r.evidence/f'{label}.ppm';r.request('emulator.framebuffer',{'path':str(original)});png(original)
   s=advance(n,native_tick-state(n)['simulation_ticks']);assert s['scene']=='Splashes' and s['display']['brightness']==15 and not s['display']['force_blank'];frame(n,label)
   dump(context.evidence/f'{label}-alignment.json',{'original':q['state'],'native':s,'reference_fields':r.request('emulator.memory',{'space':'wram','address':0xb,'length':4})})
   result=context.check(label+' exact full-bright original frame',lambda:exact(original,n.evidence/f'{label}.ppm'));observations['frames'].append({'label':label,**result})
  for scroll,original_frame in ([] if reuse else [(0x132,806),(0x142,822)]):
   q=r.request('emulator.run_fast',{'unit':'frame','count':original_frame-last,'max_operations':100000000,'timeout_ms':30000,'capture_frame':True,'capture_audio':True});assert q['complete'];last=original_frame
   m=r.request('emulator.memory',{'space':'wram','address':0x5e2,'length':2});assert int.from_bytes(bytes(m['bytes']),'little')==scroll,m
   label=f'scroll-{scroll:04x}';original=r.evidence/f'{label}.ppm';r.request('emulator.framebuffer',{'path':str(original)});png(original)
   if state(n)['scene']!='Highscores':until(n,'Highscores',600)
   qn=n.request('port.run_until',{'ticks':400,'timeout_ms':30000,'conditions':[{'pointer':'/frontend/highscores/scroll','value':scroll}]});assert qn['matched'];s=state(n);assert s['display']['scroll'][2][1]==scroll-1,s['display'];frame(n,label)
   dump(context.evidence/f'{label}-alignment.json',{'original':q['state'],'original_scroll':m,'native':s})
   result=context.check(label+' exact scrolling frame',lambda:exact(original,n.evidence/f'{label}.ppm'));observations['frames'].append({'label':label,**result})
  # Same original route provenance, selected source-ready states only. These
  # snapshots were exported from the recorded natural Name/Shop inputs, never
  # memory/register writes. Validate their named source fields on import.
  n.request('port.reset');setup_race(n,context.evidence,ready=True);advance(n,1)
  for label,original_frame,conditions in [
   ('initial-grid',1573,[{'pointer':'/transition/race/phase','value':'Delay'},{'pointer':'/transition/race/remaining_waits','value':10}]),
   ('countdown-animation',1723,[{'pointer':'/transition/race/phase','value':'CountToReady'},{'pointer':'/transition/race/countdown/state','value':13},{'pointer':'/transition/race/countdown/delay','value':4}])]:
   qn=n.request('port.run_until',{'ticks':400,'timeout_ms':30000,'conditions':conditions});assert qn['matched'];s=state(n);assert s['scene']=='Ready' and s['progression']['original_track']==4
   assert s['world']['pickup']['state']==2,'prepared spawn parity must match source nitro state2'
   snapshot=ROOT/f'debug/artifacts/original-race-resume-001/frame-{original_frame}.snapshot.json'
   q=r.request('emulator.snapshot.import',{'path':str(snapshot)});assert not q.get('artificial',False)
   pickup=r.request('emulator.memory',{'space':'wram','address':0x6ce,'length':2})['bytes'];assert pickup==[2,0],pickup
   fields=r.request('emulator.memory',{'space':'wram','address':0x77d,'length':6})['bytes'];assert fields[:4]==([0,0,6,0] if original_frame==1573 else [13,0,4,0]),fields
   # Imported original snapshots include their selected captured frame.
   original=ROOT/f'debug/artifacts/original-race-resume-001/frame-{original_frame}.ppm';frame(n,label)
   dump(context.evidence/f'{label}-alignment.json',{'original_frame':original_frame,'snapshot_sha256':hashlib.sha256(snapshot.read_bytes()).hexdigest(),'countdown':fields,'native':s,'setup_alignment':'One released native tick before NextRace confirm aligns prepared pickup parity; original CPU/DMA load crosses NMIs whereas native immutable loading is atomic. No state writes or pixel search.'})
   result=context.check(label+' exact original ready frame',lambda:exact(original,n.evidence/f'{label}.ppm'));observations['frames'].append({'label':label,**result})
  # Audio processing starts after asynchronous source upload/command handling.
  # Original NMI85 matches native83 by all8 source sequence/duration records;
  # two-frame startup handoff lag is excluded, not a tempo/pitch correction.
  r.request('emulator.reset');q=r.request('emulator.run_fast',{'unit':'frame','count':181,'max_operations':100000000,'timeout_ms':30000});assert q['complete']
  counters=r.request('emulator.memory',{'space':'wram','address':0xd,'length':1})['bytes'];assert counters==[85],counters
  n.request('port.reset');s=advance(n,83);assert s['runtime']['nmi_frames']==83
  original_start=original_audio_state(r);dump(context.evidence/'audio-start.json',{'original':original_start,'native':s,'alignment':'OriginalNMI85/native83; same sequence and note-duration phase after source command handoff'})
  context.check('original/native initial voice sequence pitch gain parity',lambda:compare_audio_state(original_start,s))
  r.request('emulator.audio.clear');n.request('port.audio.clear')
  r.request('emulator.run_fast',{'unit':'frame','count':30,'max_operations':100000000,'timeout_ms':30000,'capture_audio':True});advance(n,30)
  export=r.request('emulator.audio.export',{'path':str(context.evidence/'original-audio.wav')});assert export['dropped_stereo_frames']==0
  observations['native_audio']=audio_wav(n,context.evidence/'native-audio.wav');observations['original_audio']=export
  original_end=original_audio_state(r);end=state(n);dump(context.evidence/'audio-end.json',{'original':original_end,'native':end})
  context.check('original/native30frame sequence pitch envelope progression',lambda:compare_audio_state(original_end,end))
  context.check('native active audio at source NMI phase',lambda:observations['native_audio']['peak']>0)
  observations['presentation_review_required']=True
  return observations
