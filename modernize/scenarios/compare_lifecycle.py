"""Compare fresh faithful/modernized executions after source-rule assertions pass."""
import json
import sys
from pathlib import Path

baseline, candidate = map(Path, sys.argv[1:3])
before = json.loads((baseline/'landmarks.json').read_text())
after = json.loads((candidate/'landmarks.json').read_text())
assert before.keys() == after.keys()
for name in before:
    assert before[name]['state'] == after[name]['state'], f'{name}: semantic state changed'
    assert before[name]['frame_sha256'] == after[name]['frame_sha256'], f'{name}: frame changed'
a = json.loads((baseline/'checkpoint.json').read_text())
b = json.loads((candidate/'checkpoint.json').read_text())
# Builds legitimately have different identities; both runs validate their own
# recipes normally. Never import a checkpoint across identities or rewrite it.
for key in a:
    if key != 'identity':
        assert a[key] == b[key], f'checkpoint {key} changed'
result = {'passed': True, 'source_rule_assertions': 'both lifecycle result.json files pass', 'exact_semantic_and_frame_landmarks': list(before), 'checkpoint_fields_exact_except_build_identity': True, 'frame_sha256': b['frame_sha256'], 'audio_sha256': b['audio_sha256'], 'baseline_identity':a['identity'], 'candidate_identity':b['identity'], 'scope':'bounded natural purchase/consume frontend/race route, including retained input history, yield timing and complete-route PCM hash; not whole-game equivalence'}
(candidate/'baseline-comparison.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result),flush=True)
