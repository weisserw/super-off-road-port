"""Common QA-harness adapter for the source-backed iteration scenario."""
import json
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent))
from lifecycle import ROOT, run as run_lifecycle


def run(context, entry):
    if entry['id'] != 'native-inventory-timer-frontend':
        raise ValueError('unknown modernization scenario')
    fresh = context.evidence/'fresh'
    with context.phase('setup'):
        binary = ROOT/'port/target/release/super-off-road-port'
        assert binary.is_file()
    with context.phase('execution'):
        context.check('natural source-rule lifecycle with branching and guarded replay',
                      lambda: run_lifecycle(binary, fresh))
    result = json.loads((fresh/'result.json').read_text())
    result['presentation_review_required'] = True
    return result
