import sys,json
from pathlib import Path
sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
root=Path('/root/super-off-road');e=root/'debug/artifacts/reference-audio-001'
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(root/'input/game.rom')],seconds=90,max_requests=12,evidence=e) as p:
 p.request('emulator.start');q=p.request('emulator.run_fast',{'unit':'frame','count':181,'max_operations':100000000,'timeout_ms':30000,'capture_frame':True});assert q['complete'];m=p.request('emulator.memory',{'space':'wram','address':0xb,'length':4});p.request('emulator.audio.clear');q=p.request('emulator.run_fast',{'unit':'frame','count':30,'max_operations':100000000,'timeout_ms':30000,'capture_audio':True});assert q['complete'];r=p.request('emulator.audio.export',{'path':str(e/'original.wav')});print(m,r,flush=True);(e/'alignment.json').write_text(json.dumps({'starting_fields':m,'result':r,'frames':q},indent=2))
