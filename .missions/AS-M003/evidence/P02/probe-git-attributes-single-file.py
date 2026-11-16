"""Identify attribute conversion in the actual read-only Git inventory commands."""
from pathlib import Path
import json,os,subprocess,tempfile
observations=[]
with tempfile.TemporaryDirectory() as directory:
 root=Path(directory)
 env=dict(os.environ,GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_NO_LAZY_FETCH='1',GIT_OPTIONAL_LOCKS='0')
 def git(*args):
  return subprocess.run(['git','-C',directory,*args],env=env,capture_output=True)
 assert git('init','-q').returncode==0
 (root/'a.ts').write_text('head\n');assert git('add','.').returncode==0
 assert git('-c','user.name=Probe','-c','user.email=probe@example.invalid','commit','-qm','head').returncode==0
 (root/'a.ts').write_text('index\n');assert git('add','.').returncode==0
 (root/'a.ts').write_text('working\n');(root/'.gitattributes').write_text('*.ts filter=probe diff=probe\n')
 for key in ['filter.probe.clean','filter.probe.smudge','filter.probe.process','diff.probe.textconv','diff.probe.command','diff.external','core.fsmonitor']:
  name=key.replace('.','-');command=f'touch {root/name}; '+('exit 1' if key.endswith('process') else 'cat')
  assert git('config',key,command).returncode==0
 empty=git('hash-object','-t','tree','--stdin').stdout.decode().strip()
 query=git('cat-file','-t',empty);observations.append({'case':'virtual-empty-tree','oid':empty,'exit':query.returncode,'stdout':query.stdout.decode(),'stderr':query.stderr.decode()})
 flags=['--raw','--ita-invisible-in-index','--no-renames','--no-relative','--no-ext-diff','--no-textconv','--ignore-submodules=none','--abbrev=64','-z']
 for attribute_source in [None,empty]:
  for command in [['show-ref'],['ls-files','-v','-z'],['ls-files','--others','--exclude-standard','-z'],['diff-files',*flags,'--'],['diff-index',*flags,'HEAD','--']]:
   for marker in root.glob('filter-*'):marker.unlink()
   for marker in root.glob('diff-*'):marker.unlink()
   prefix=['-c','core.fsmonitor=false','-c','core.filemode=true','-c','core.hooksPath=/dev/null','-c','diff.external=']
   if attribute_source: prefix+=['--attr-source='+attribute_source]
   result=git(*prefix,*command)
   observations.append({'attributeSource':attribute_source,'command':command,'exit':result.returncode,'helpers':[p.name for pattern in ['filter-*','diff-*','core-*'] for p in root.glob(pattern)],'stderr':result.stderr.decode(),'inventoryBytes':len(result.stdout)})
Path(__file__).with_suffix('.json').write_text(json.dumps(observations,indent=2)+'\n')
print(json.dumps(observations,indent=2))
