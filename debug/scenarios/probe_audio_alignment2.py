"""Named initial audio uncertainty: same NMI85, compare source pitch/sequence and PCM.
Stop after30source/native frames; no gameplay progression or synthetic state edits.
"""
import sys,json
from pathlib import Path
sys.path.insert(0,'/root/romatlas/scripts');sys.path.insert(0,str(Path(__file__).resolve().parent))
from native_debug_client import PortClient
from initial_adapter import advance,state,audio_wav,dump
root=Path('/root/super-off-road');e=root/'debug/artifacts/audio-alignment-002';e.mkdir()
with PortClient([str(root/'port/target/release/super-off-road-port'),'--rom',str(root/'input/game.rom'),'--debug-stdio'],seconds=90,max_requests=30,evidence=e/'native') as n:
 advance(n,83);dump(e/'native-start.json',state(n));n.request('port.audio.clear');advance(n,30);dump(e/'native-end.json',state(n));print('native',audio_wav(n,e/'native.wav'),flush=True)
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(root/'input/game.rom')],seconds=90,max_requests=30,evidence=e/'original') as r:
 r.request('emulator.start');q=r.request('emulator.run_fast',{'unit':'frame','count':181,'max_operations':100000000,'timeout_ms':30000});assert q['complete']
 def observed():
  out={}
  for name,address,length in [('dsp_pitch',0x10,16),('instrument_gain',0x20,16),('volume',0,16),('sequence',0x30,16),('duration',0x80,16),('envelope',0x90,16),('pitch',0x260,16),('master_song',0xdb,5),('heartbeat',0xd5,1)]:out[name]=r.request('emulator.memory',{'space':'aram','address':address,'length':length})['bytes']
  return out
 dump(e/'original-start.json',observed());r.request('emulator.audio.clear');q=r.request('emulator.run_fast',{'unit':'frame','count':30,'max_operations':100000000,'timeout_ms':30000,'capture_audio':True});assert q['complete'];dump(e/'original-end.json',observed());print('original',r.request('emulator.audio.export',{'path':str(e/'original.wav')}),flush=True)
