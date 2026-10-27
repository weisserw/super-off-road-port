"""Fresh natural-input inventory/timer/frontend regression; no state writes.

Expectations: source consume_nitro_on_press_edge (game.asm:6056), shop
purchase/normalization (shop.trace.md), timer chain (game.asm:14734), and
frontend return/yield contracts (names.trace.md/shop.trace.md). Baseline hashes
are additional differential evidence, never a source of success expectations.
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, '/root/romatlas/scripts')
from native_debug_client import PortClient
from scenario_client import ScenarioSession


def packed(value):
    return (value // 10) * 16 + value % 10


def units(timer):
    def dec(value):
        assert value >> 4 <= 9 and value & 15 <= 9
        return (value >> 4) * 10 + (value & 15)
    return dec(timer['high_bcd']) * 10000 + dec(timer['middle_bcd']) * 100 + dec(timer['fraction_bcd'])


def run(binary, output):
    output.mkdir(parents=True, exist_ok=False)
    with PortClient([str(binary), '--rom', str(ROOT/'input/game.rom'), '--debug-stdio', '--rng-seed', '0000000000'], seconds=180, max_requests=600, evidence=output/'transport') as client:
        session = ScenarioSession(client, native=True)
        session.reset()
        landmarks = {}
        def state():
            value = client.request('port.state')['game']
            assert value['flags']['fault'] is None, value['flags']
            return value
        def mark(label):
            value = state()
            frame = client.request('port.framebuffer')
            landmarks[label] = {'state': value, 'frame_sha256': frame['sha256']}
            client.frame(label+'.ppm')
            return value
        def advance(count):
            session.advance(count)
            return state()
        def until(scene):
            session.until(conditions=[{'pointer': '/scene', 'value': scene}], ticks=1000, timeout_ms=30000)
            return state()
        def press(actions, count=8, release=4):
            session.input(actions)
            advance(count)
            session.input([])
            return advance(release)
        advance(340)
        session.input(['start'])
        until('Names')
        session.input([])
        advance(100)
        assert state()['transition']['menu']['Names']['phase'] == 'WaitFrame'
        press(['select'], 1, 50)
        press(['up'])
        assert state()['transition']['menu']['Names']['editors'][0]['row'] == 6
        press(['b'])
        until('Shop')
        advance(100)
        shop = mark('shop-before-purchase')
        assert shop['transition']['menu']['Shop']['phase'] == 'WaitFrame'
        editor = shop['transition']['menu']['Shop']['selection']['editors'][0]
        assert (editor['row'], editor['column']) == (0, 0), editor
        assert shop['actors'][0]['nitro'] == 0
        session.bookmark('before-purchase')
        press(['b'], 1, 12)
        bought = mark('shop-after-purchase')
        assert bought['actors'][0]['nitro'] == 1
        # Rewind the branch and re-execute the purchase. Only the retained branch
        # belongs to the durable recipe; this exercises isolated snapshots too.
        session.restore('before-purchase')
        press(['b'], 1, 12)
        assert state() == bought
        press(['down'], 25)
        press(['right'], 4)
        press(['b'])
        start = until('Racing')
        mark('racing-stock-one')
        assert start['actors'][0]['nitro'] == 1
        before = units(start['timers']['race'])
        session.input(['y', 'b'])
        sampled = advance(1)
        # Movement consumes the previous service's retained controller word;
        # source input copy happens after movement, feeding the next service.
        assert sampled['actors'][0]['nitro'] == 1
        assert units(sampled['timers']['race']) == before + 16
        used = advance(1)
        assert used['actors'][0]['nitro'] == 0
        assert units(used['timers']['race']) == before + 2 * 16
        held = advance(8)
        assert held['actors'][0]['nitro'] == 0
        assert units(held['timers']['race']) == before + 10 * 16
        session.input([])
        advance(1)
        session.input(['y'])
        empty = advance(1)
        assert empty['actors'][0]['nitro'] == 0
        end = mark('driven-empty-stock')
        recipe = session.export_recipe()
        (output/'recipe.json').write_text(json.dumps(recipe, indent=2)+'\n')
        checkpoint = client.request('port.checkpoint.export')
        (output/'checkpoint.json').write_text(json.dumps(checkpoint, indent=2)+'\n')
        # Replay the exact successful branch afresh, retaining normal identity
        # checks. This is setup validation, not cached assertion success.
        session.replay(recipe)
        assert state() == end
        (output/'landmarks.json').write_text(json.dumps(landmarks, indent=2)+'\n')
        result = {'passed': True, 'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'rom_sha256': hashlib.sha256((ROOT/'input/game.rom').read_bytes()).hexdigest(), 'identity': client.request('api.describe')['identity'], 'scope': 'natural joining, name exit, nitro purchase, shop exit, ready, press/held/empty consumption, timer progression, snapshot branch and guarded recipe replay'}
        (output/'result.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps(result), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    run(args.binary.resolve(), args.output.resolve())
