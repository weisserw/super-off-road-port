"""Independent original inventory/timer observation from a validated natural shop.

Origin was produced by debug/scenarios/probe_original_race.py using only inputs,
from the natural frame806 highscore cache. No emulator state writes are used.
The retained origin and suffix recipe are separate; neither is represented as a
reset-only recipe. Re-exported snapshots/captures stay local.
"""
import hashlib
import json
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, '/root/romatlas/scripts')
from native_debug_client import PortClient
from scenario_client import ScenarioSession

class ReferenceSession(ScenarioSession):
    def advance(self, count, *, unit='frame'):
        assert unit == 'frame' and 1 <= count <= 1000
        result = self._mutate('emulator.run_fast', {'unit':'frame','count':count,'max_operations':100000000,'timeout_ms':30000,'capture_frame':True})
        assert result['complete'], result
        self.recipe.append({'kind':'advance','count':count,'unit':'frame'})
        return result


def run(output):
    output.mkdir(parents=True, exist_ok=False)
    snapshot = ROOT/'debug/artifacts/original-route-001/shop.snapshot.json'
    producer = ROOT/'debug/scenarios/probe_original_race.py'
    metadata = ROOT/'debug/artifacts/original-route-001/shop.json'
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    assert sha(snapshot) == 'f67d6775f5683bdc1c4f74fedaa5ebeb5125ebdc4162a1ccd97458b3fa690799', 'validated original shop cache changed'
    assert json.loads(snapshot.read_text())['artificial'] is False
    assert json.loads(metadata.read_text())['frames'] == 1378
    with PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','snes','--rom',str(ROOT/'input/game.rom')], seconds=240, max_requests=100, evidence=output/'transport') as client:
        api = client.request('api.describe')
        client.request('emulator.start')
        assert client.request('rom.info')['sha256'] == '24b687f95bb9737d223738f13c00aeaa7d697fa9e2fd50597b81d0cfa2160daa'
        restored = client.request('emulator.snapshot.import', {'path':str(snapshot)})
        assert not restored.get('artificial', False)
        session = ReferenceSession(client, native=False)
        def read(address, length):
            return client.request('emulator.memory', {'space':'wram','address':address,'length':length})['bytes']
        def press(actions, count=8, release=4):
            session.input(actions);session.advance(count);session.input([]);session.advance(release)
        assert read(0x61b, 1) == [0]
        session.bookmark('before-purchase')
        press(['b'], 1, 12)
        assert read(0x61b, 1) == [1]
        session.restore('before-purchase')
        press(['b'], 1, 12)
        assert read(0x61b, 1) == [1]
        press(['down'],25);press(['right'],4);press(['b'])
        session.advance(600)
        assert read(0x61b,1) == [1]
        before = read(0x6c3,3)
        session.input(['y','b']);session.advance(8)
        used = read(0x61b,1)
        after = read(0x6c3,3)
        assert used == [0]
        def units(timer):
            values = [(b >> 4) * 10 + (b & 15) for b in timer]
            return values[0] + values[1]*100 + values[2]*10000
        assert units(after) - units(before) == 8*16, (before, after)
        session.advance(8)
        assert read(0x61b,1) == [0]
        client.request('emulator.framebuffer', {'path':str(output/'original-after-consumption.ppm')})
        client.request('emulator.snapshot.export', {'path':str(output/'original-after-consumption.snapshot.json')})
        recipe = {'origin':{'snapshot':str(snapshot.relative_to(ROOT)),'sha256':sha(snapshot),'natural_producer':str(producer.relative_to(ROOT)),'producer_sha256':sha(producer),'metadata_sha256':sha(metadata)},'steps':session.recipe,'api':api,'rom_sha256':sha(ROOT/'input/game.rom'),'frozen_source_sha256':sha(ROOT/'reference/source/game.asm')}
        (output/'recipe.json').write_text(json.dumps(recipe,indent=2)+'\n')
        result = {'passed':True,'stock_before':1,'stock_after':used[0],'held_preserves_empty':True,'timer_before':before,'timer_after_eight_frames':after,'elapsed_source_units':128,'scope':'original natural shop purchase→race→consume/hold and per-frame timer progression; no overflow/invalid-BCD hardware claim'}
        (output/'result.json').write_text(json.dumps(result,indent=2)+'\n')
        print(json.dumps(result),flush=True)

if __name__ == '__main__':
    run(Path(sys.argv[1]).resolve())
