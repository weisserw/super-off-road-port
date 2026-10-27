import sys,json
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent));sys.path.insert(0,'/root/romatlas/scripts')
from native_debug_client import PortClient
from initial_adapter import advance,state,dump,audio_wav
root=Path('/root/super-off-road');e=root/'debug/artifacts/native-audio-phase-001';e.mkdir()
with PortClient([str(root/'port/target/release/super-off-road-port'),'--rom',str(root/'input/game.rom'),'--debug-stdio'],seconds=90,max_requests=30,evidence=e/'native') as p:
 advance(p,83);dump(e/'start.json',state(p));p.request('port.audio.clear');advance(p,30);dump(e/'end.json',state(p));print(audio_wav(p,e/'native.wav'),flush=True)
for endpoint in ['start','end']:
 n=json.load(open(e/f'{endpoint}.json'))['audio'];o=json.load(open(root/f'debug/artifacts/audio-alignment-001/original-{endpoint}.json'));print(endpoint,'native',[(v['sequence'],v['duration'],v['pitch'])for v in n['voices']],'original',[(int.from_bytes(bytes(o['sequence'][i:i+2]),'little'),o['duration'][i],int.from_bytes(bytes(o['pitch'][i:i+2]),'little'))for i in range(0,16,2)],flush=True)
