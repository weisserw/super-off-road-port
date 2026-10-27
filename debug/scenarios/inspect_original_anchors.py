import sys,json
from pathlib import Path
sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
root=Path('/root/super-off-road');e=root/'debug/artifacts/original-anchors-001'
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(root/'input/game.rom')],seconds=60,max_requests=30,evidence=e) as p:
 p.request('emulator.start');out={}
 for f in [1573,1723]:
  p.request('emulator.snapshot.import',{'path':str(root/f'debug/artifacts/original-race-resume-001/frame-{f}.snapshot.json')});values={}
  for name,address,size in [('nmi_animation',0xb,4),('window',0x549,14),('countdown',0x77d,6),('truck_progress',0x4d1,12)]:values[name]=p.request('emulator.memory',{'space':'wram','address':address,'length':size})['bytes']
  out[f]=values;print(f,values,flush=True)
 (e/'anchors.json').write_text(json.dumps(out,indent=2))
