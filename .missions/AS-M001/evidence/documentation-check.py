from pathlib import Path
import re, json, subprocess, hashlib, sys
ROOT=Path(__file__).resolve().parents[3]
issues=[]; checked=0
changed=subprocess.check_output(['git','diff','--name-only'],cwd=ROOT,text=True).splitlines()
untracked=subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=ROOT,text=True).splitlines()
for name in changed+untracked:
 if not (name.startswith('.missions/AS-M001/') or name.startswith('docs/missions/AS-M001/') or name in ['docs/PRD.md','docs/IMPLEMENTATION_ROADMAP.md','README.md']): issues.append('Out-of-scope file: '+name)
 p=ROOT/name
 if p.suffix!='.md': continue
 for target in re.findall(r'\[[^\]\n]*\]\(([^)]+)\)',p.read_text()):
  target=target.split(' "')[0].strip('<>')
  if re.match(r'^[a-zA-Z][\w+.-]*:',target): continue
  path,_,anchor=target.partition('#'); dest=p.parent/path if path else p
  checked+=1
  if not dest.exists(): issues.append(f'{name}: missing {target}'); continue
  if anchor and dest.suffix=='.md':
   headings=[]
   for h in re.findall(r'^#{1,6}\s+(.+)$',dest.read_text(),re.M):
    h=re.sub(r'\[([^]]+)\]\([^)]*\)',r'\1',h)
    headings.append(re.sub(r'[^\w\- ]','',h.lower()).replace(' ','-'))
   if anchor not in headings: issues.append(f'{name}: missing anchor {target}')
base='4f16dca764a6037176f5796e50d8e25db523dd92'
for name in ['docs/missions/AS-M001/r1.md']:
 original=subprocess.check_output(['git','show',base+':'+name],cwd=ROOT)
 if (ROOT/name).read_bytes()!=original: issues.append('Historical contract changed: '+name)
road='docs/IMPLEMENTATION_ROADMAP.md'; marker=b'## Retained 2026-09-22 plan'
original=subprocess.check_output(['git','show',base+':'+road],cwd=ROOT).split(marker,1)[1]
if (ROOT/road).read_bytes().split(marker,1)[1]!=original: issues.append('Retained roadmap changed')
history='.missions/AS-M001/history.jsonl'; original=subprocess.check_output(['git','show',base+':'+history],cwd=ROOT)
if not (ROOT/history).read_bytes().startswith(original): issues.append('History prefix changed')
initial=json.loads((ROOT/'.missions/AS-M001/evidence/initial-coordination.json').read_text())['trackedFiles']
protected=[p for p in initial if not (p.startswith('.missions/AS-M001/') or p in ['docs/PRD.md','docs/IMPLEMENTATION_ROADMAP.md','README.md'])]
for name in protected:
 p=ROOT/name
 if not p.is_file() or hashlib.sha256(p.read_bytes()).hexdigest()!=initial[name]: issues.append('Protected file changed: '+name)
print(json.dumps(dict(valid=not issues,localLinksChecked=checked,protectedFilesChecked=len(protected),historicalContractAndRoadmapPreserved=True if not any('Historical' in i or 'Retained' in i for i in issues) else False,issues=issues),indent=2))
sys.exit(bool(issues))
