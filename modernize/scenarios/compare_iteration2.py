"""Additional differential evidence after independently source-backed checks pass."""
import hashlib,json,sys
from pathlib import Path
before,after=map(Path,sys.argv[1:3])
def load(path):return json.loads(path.read_text())
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
assert load(before/'result.json')['status']=='passed'
assert load(after/'result.json')['status']=='passed'
a,b=load(before/'landmarks.json'),load(after/'landmarks.json')
assert a.keys()==b.keys()
for name in a:assert a[name]==b[name],f'{name}: semantic state changed'
frames={}
for name in ['response.ppm','depleted.ppm']:
 assert digest(before/'native'/name)==digest(after/'native'/name),f'{name}: pixels changed'
 frames[name]=digest(after/'native'/name)
a,b=load(before/'response-checkpoint.json'),load(after/'response-checkpoint.json')
for key in a:
 if key!='identity':assert a[key]==b[key],f'checkpoint {key} changed'
result={'passed':True,'exact_semantic_landmarks':['response','depleted','lap'],'exact_full_frames':frames,'whole_route_pcm_sha256':b['audio_sha256'],'checkpoint_fields_identical_except_build_identity':True,'baseline_identity':a['identity'],'candidate_identity':b['identity'],'scope':'natural attract track6 completion, track2 hazard lifecycle/AI lap and guarded replay; separate source-derived contracts cover redirects and conditional states'}
(after/'baseline-comparison.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
