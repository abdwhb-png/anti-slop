"""Match the real helper regression fixture, including unchanged staged-only.ts."""
from pathlib import Path
import json,os,subprocess,tempfile
observations=[]
for location in ['working', 'info']:
 with tempfile.TemporaryDirectory() as directory:
  root=Path(directory)
  env=dict(os.environ,GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_NO_LAZY_FETCH='1',GIT_OPTIONAL_LOCKS='0',GIT_ATTR_NOSYSTEM='1')
  def git(*args):return subprocess.run(['git','-C',directory,*args],env=env,capture_output=True)
  assert git('init','-q').returncode==0
  (root/'a.ts').write_text('head\n');(root/'staged-only.ts').write_text('old\n');assert git('add','.').returncode==0
  assert git('-c','user.name=Probe','-c','user.email=probe@example.invalid','commit','-qm','head').returncode==0
  (root/'a.ts').write_text('index\n');(root/'staged-only.ts').write_text('staged-only\n');assert git('add','.').returncode==0
  (root/'a.ts').write_text('working\n')
  (root/('.gitattributes' if location=='working' else '.git/info/attributes')).write_text('*.ts filter=probe diff=probe\n')
  for key in ['filter.probe.clean','filter.probe.smudge','filter.probe.process','diff.probe.textconv','diff.probe.command','diff.external','core.fsmonitor']:
   command=f'touch {root/"helper-ran-"}{key}; cat'
   assert git('config',key,command).returncode==0
  empty=git('hash-object','-t','tree','--stdin').stdout.decode().strip()
  flags=['--raw','--ita-invisible-in-index','--no-renames','--no-relative','--no-ext-diff','--no-textconv','--ignore-submodules=none','--abbrev=64','-z']
  for attribute_source in [None,empty]:
   for command in [['diff-files',*flags,'--'],['diff-index',*flags,'HEAD','--']]:
    for marker in root.glob('helper-ran-*'):marker.unlink()
    prefix=['-c','core.fsmonitor=false','-c','core.filemode=true','-c','core.hooksPath=/dev/null','-c','diff.external=','-c','core.attributesFile=/dev/null']
    if attribute_source:prefix+=['--attr-source='+attribute_source]
    result=git(*prefix,*command)
    observations.append({'attributeLocation':location,'attributeSource':attribute_source,'command':command,'exit':result.returncode,'helpers':[p.name for p in root.glob('helper-ran-*')],'stderr':result.stderr.decode(),'inventoryBytes':len(result.stdout)})
Path(__file__).with_suffix('.json').write_text(json.dumps(observations,indent=2)+'\n')
print(json.dumps(observations,indent=2))
