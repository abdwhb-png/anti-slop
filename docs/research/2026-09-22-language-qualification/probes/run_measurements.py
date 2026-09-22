"""Run controlled measurements sequentially; generated fixtures are not production benchmarks."""
from pathlib import Path
import json, platform, sys
from measure import run

root=Path(__file__).resolve().parent
output=Path(sys.argv[1]).resolve();output.mkdir(exist_ok=False)
typescript=Path(sys.argv[2]).resolve()
results=[]
for size in ['small','medium']:
    fixture=root/('fixtures-'+size)
    paths=sorted((fixture/'ts').rglob('*.ts'))
    cases={
        'oxc':[str(root/'oxc-probe/target/release/anti_slop_oxc_probe'),*[str(p) for p in paths]],
        'mago-native':[str(root/'bin/mago'),'--workspace',str(fixture/'php'),'analyze','--no-extensions','--reporting-format','json'],
        'mago-sdk':[str(root/'bin/mago'),'--workspace',str(fixture/'php'),'analyze','--reporting-format','json'],
    }
    for name,command in cases.items():
        for i in range(10):
            result=run(command,output/f'{size}-{name}-{i:02}',root)
            result.update({'workload':size,'analyzer':name,'iteration':i})
            results.append(result)
            if result['exit']!=0:raise RuntimeError(f'{name}: inspect stderr in {output}')
        print(json.dumps({'completed':name,'workload':size}),flush=True)
    for i in range(3):
        command=['node',str(root/'typescript_probe.cjs'),str(typescript),str(fixture/'ts')]
        result=run(command,output/f'{size}-typescript-{i:02}',root)
        result.update({'workload':size,'analyzer':'typescript','iteration':i})
        results.append(result)
        if result['exit']!=0:raise RuntimeError('TypeScript failed')
    print(json.dumps({'completed':'typescript','workload':size}),flush=True)
(output/'measurements.json').write_text(json.dumps({'host':platform.platform(),'results':results},indent=2)+'\n')
