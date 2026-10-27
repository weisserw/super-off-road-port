"""Compare pickup runs after their independent source-rule assertions pass."""
import argparse,hashlib,json
from pathlib import Path

def load(path):return json.loads(path.read_text())
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
parser=argparse.ArgumentParser();parser.add_argument('before',type=Path);parser.add_argument('after',type=Path);parser.add_argument('--original',type=Path,required=True);args=parser.parse_args()
for p in [args.before,args.after,args.original]:assert load(p/'result.json')['passed']
a,b=[load(p/'landmarks.json')for p in [args.before,args.after]]
assert a==b,'pickup semantic landmarks changed'
x,y=[load(p/'checkpoint.json')for p in [args.before,args.after]]
for key in x:
 if key!='identity':assert x[key]==y[key],f'checkpoint {key} changed'
frames={}
for name in ['collected.ppm','expired.ppm']:
 assert sha(args.before/'transport'/name)==sha(args.after/'transport'/name),f'frame {name} changed'
 frames[name]=sha(args.after/'transport'/name)
original=load(args.original/'landmarks.json');n=b['collected'];o=original['collected'];p=n['world']['pickup'];before=load(args.original/'before-collection.json')
# Source physical4670: use old count, saturate count4, byte award units.
assert before['state']==1 and before['counts'][0]==0
assert p['state']==o['state']==7 and p['counts'][0]==o['counts'][0]==1
assert p['placement']['world']==o['world'] and p['placement']['height']==o['height']
assert [s['tile_attributes']for s in p['sprites']]==[s['tile_attributes']for s in o['sprites']]==[0x36e0,0x36ce]
assert n['world']['pending_awards'][n['runtime']['animation_frame']&3]==1
assert o['pending_awards'][o['animation']&3]==1
# Source46C5: every-fourth-frame rise is applied immediately on collection.
# Compare projection after accounting for this documented phase difference;
# do not mask pixels, edit state or claim two whole race frames are aligned.
for ns,os in zip(p['sprites'],o['sprites']):
 nu=(ns['coordinates']+(0x100 if n['runtime']['animation_frame']&3==0 else 0))&65535
 ou=(os['coordinates']+(0x100 if o['animation']&3==0 else 0))&65535
 assert nu==ou
for point,remaining in [('collected',79),('last',1),('expired',0)]:
 assert b[point]['world']['pickup']['lifetime']==original[point]['lifetime']==remaining
assert original['expired']['state']==0
result={'passed':True,'exact_native_semantic_landmarks':list(a),'exact_native_frames':frames,'exact_native_route_pcm_sha256':y['audio_sha256'],'checkpoint_fields_exact_except_identity':True,'original_money_contract':['old_count0→state7/one pending unit for rotating collector','count1','same placement/art and projected pair before phase-dependent rise','lifetime79→1→0 and hidden expiry'],'original_alignment_limits':'Natural startup/loading differs: native frame101 collector1 vs original frame64 collector0, with one extra source rise on collection; original has already collected a nitro pickup. Whole race frames and cumulative state are not asserted equal.','source':'reference/source/game.asm physical45C7..48DB and objects.trace.md'}
(args.after/'comparison.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
