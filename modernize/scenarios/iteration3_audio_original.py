"""Independent original initial music and natural nitro effect observations.

Source phase alignment is established by initial_sanity_adapter.py; source
shop cache provenance is guarded. Captures stay local; no register/memory writes.
"""
from pathlib import Path
import sys,json,hashlib
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=['/root/romatlas/scripts',str(ROOT/'debug/scenarios'),str(ROOT/'modernize/scenarios')]
from native_debug_client import PortClient
from original_inventory import ReferenceSession
from initial_sanity_adapter import original_audio_state
out=Path(sys.argv[1]).resolve();out.mkdir(exist_ok=False)
with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(ROOT/'input/game.rom')],seconds=240,max_requests=180,evidence=out/'transport') as c:
 c.request('emulator.start');assert c.request('rom.info')['sha256']=='24b687f95bb9737d223738f13c00aeaa7d697fa9e2fd50597b81d0cfa2160daa'
 s=ReferenceSession(c,native=False);s.reset();s.advance(181)
 before=original_audio_state(c);assert c.request('emulator.memory',{'space':'wram','address':0xd,'length':1})['bytes']==[85]
 s.advance(30);after=original_audio_state(c)
 (out/'music.json').write_text(json.dumps({'before':before,'after':after,'recipe':s.recipe,'alignment':'Original frames181/211, NMI85/115; native ticks83/113 from established source command-handoff alignment.'},indent=2)+'\n')
 origin=ROOT/'debug/artifacts/original-route-001/shop.snapshot.json';assert hashlib.sha256(origin.read_bytes()).hexdigest()=='f67d6775f5683bdc1c4f74fedaa5ebeb5125ebdc4162a1ccd97458b3fa690799'
 c.request('emulator.snapshot.import',{'path':str(origin)});s=ReferenceSession(c,native=False)
 def press(actions,count=8,release=4):s.input(actions);s.advance(count);s.input([]);s.advance(release)
 press(['b'],1,12);press(['down'],25);press(['right'],4);press(['b']);s.advance(600)
 s.input(['y','b']);s.advance(8);fx=original_audio_state(c)
 assert c.request('emulator.memory',{'space':'wram','address':0x61b,'length':1})['bytes']==[0]
 s.input([]);s.advance(2);later=original_audio_state(c)
 (out/'effect.json').write_text(json.dumps({'before':fx,'after':later,'origin':str(origin.relative_to(ROOT)),'origin_sha256':hashlib.sha256(origin.read_bytes()).hexdigest(),'recipe':s.recipe,'effect_id':2},indent=2)+'\n')
 print('music',before,after,flush=True);print('effect',fx,later,flush=True)

 assert fx['sequence'][6]==later['sequence'][6]==0x4365
 assert fx['sample'][6]==later['sample'][6]==23
 assert [fx['duration'][6],later['duration'][6]]==[109,105]
 assert [fx['pitch'][6],later['pitch'][6]]==[879,1023]
 assert [fx['amplitude'][6],later['amplitude'][6]]==[81,78]
 (out/'result.json').write_text(json.dumps({'passed':True,'scope':'Fresh original initial8-voice music states30frames apart and natural nitro effect2 states2frames apart. Guarded natural shop origin; no state writes.','source':'frozen game.asm SPC effect2 prefix/voice update and audio.trace.md','limits':'Original/native PCM interpolation differs; exact original PCM is not asserted.'},indent=2)+'\n')
