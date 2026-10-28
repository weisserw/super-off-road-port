"""Original terrain input/output observations at natural wheel-call boundaries.

PC9678 calls the terrain sampler; PC967B is its return before PLY. Reads
coordinates/history before and A/carry/history after, with no state writes.
"""
from pathlib import Path
import sys,json,hashlib
ROOT=Path('/root/super-off-road');sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
sys.path.insert(0,str(ROOT/'modernize/scenarios'))
from original_inventory import ReferenceSession
out=Path(sys.argv[1]).resolve();out.mkdir(exist_ok=False)
origin=ROOT/'modernize/evidence/iteration-003/M6-original-contact/contact.snapshot.json'
assert json.loads(origin.read_text())['artificial'] is False
assert hashlib.sha256(origin.read_bytes()).hexdigest()=='a5c22e7e46e5a1d72b64664143e34e80adafe1a5db352780c90501fb9f1af4e4'
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(ROOT/'input/game.rom')],seconds=300,max_requests=1500,evidence=out/'transport') as c:
 c.request('emulator.start');c.request('emulator.snapshot.import',{'path':str(origin)});s=ReferenceSession(c,native=False)
 def read(a,n):return c.request('emulator.memory',{'space':'wram','address':a,'length':n})['bytes']
 def until(pc):
  return s.until(queries=[{'name':'cpu','method':'emulator.state','params':{}}],conditions=[{'query':'cpu','pointer':'/main/pc','value':pc},{'query':'cpu','pointer':'/main/mid_instruction','value':False}],max_operations=1000000,timeout_ms=30000)
 records=[]
 for i in range(96):
  if i and i%16==0:s.advance(180)
  until(0x9678)
  q=read(0x43,4);track=read(0x1b,1)[0];before=read(0x65,1)[0];pos=[int.from_bytes(bytes(q[j:j+2]),'little') for j in [0,2]]
  if i==0:s.bookmark('probe')
  until(0x967b);state=c.request('emulator.state')['main'];after=read(0x65,1)[0]
  records.append({'position':pos,'track':track,'history_before':before,'history_after':after,'blocked':bool(state['p']&1),'value':state['a']&255})
  if i%16==15:print('probes',i+1,'heights',sorted({r['value'] for r in records if not r['blocked']}),'blocked',sum(r['blocked'] for r in records),flush=True)
 # Read-only identity check of the actual copied terrain, not a synthetic setup.
 starts=[0x20000,0x216b0,0x22d60,0x24410,0x28000,0x296b0,0x2ad60,0x2c410,0x50000,0x516aa,0x52d5a,0x5440a,0x58000,0x596b0,0x5ad60,0x5c410]
 raw=bytes(read(0x10000,4096)+read(0x11000,1713));rom=(ROOT/'input/game.rom').read_bytes();assert raw==rom[starts[track]:starts[track]+5809]
 recipe={'origin':str(origin.relative_to(ROOT)),'origin_sha256':hashlib.sha256(origin.read_bytes()).hexdigest(),'natural_producer':'modernize/scenarios/iteration3_contacts.py original','steps':s.recipe}
 (out/'recipe.json').write_text(json.dumps(recipe,indent=2)+'\n')
 s.restore('probe');until(0x967b);q=c.request('emulator.state')['main'];assert (q['a']&255,bool(q['p']&1),read(0x65,1)[0])==(records[0]['value'],records[0]['blocked'],records[0]['history_after'])
 (out/'observations.json').write_text(json.dumps(records,indent=2)+'\n')
 (out/'result.json').write_text(json.dumps({'passed':True,'scope':'96 natural original wheel sampler calls across900 further race frames, captured input coordinates/history and actual A/carry/history output; copied terrain matches ROM, persistent first-call branch reproduced. No state writes.','copied_terrain_sha256':hashlib.sha256(raw).hexdigest()},indent=2)+'\n')
