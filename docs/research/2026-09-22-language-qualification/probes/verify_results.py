"""Assertions against real analyzer results and known fixture facts; no mocked analyzer."""
from pathlib import Path
import json,sys

root=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else Path(__file__).resolve().parent
results=root/'measurements';checks=[]
def check(name,condition):checks.append({'id':name,'passed':bool(condition)})
def read(path):return json.loads(path.read_text())
def fixture_source(recorded_path):
    # Raw observations retain their original absolute paths. Read only the
    # corresponding archived fixture when checking a relocated evidence bundle.
    parts=Path(recorded_path).parts
    start=next(i for i,part in enumerate(parts) if part in {'fixtures-small','fixtures-medium'})
    return root.joinpath(*parts[start:]).read_bytes()
for size,files in [('small',3),('medium',203)]:
    oxc=read(results/f'{size}-oxc-final-00.stdout')
    check(f'oxc-{size}-all-selected-files',len(oxc['files'])==files)
    check(f'oxc-{size}-no-parse-errors',all(not f['parse_errors'] for f in oxc['files']))
    consumer=next(f for f in oxc['files'] if f['file'].endswith('/service.ts'))
    check(f'oxc-{size}-alias-local-reference',any(s['name']=='parseId' and any(r['text']=='parseId' for r in s['references']) for s in consumer['symbols']))
    check(f'oxc-{size}-dynamic-import-node',any(n['kind']=='dynamic_import' and n['text']=='import(path)' for n in consumer['nodes']))
    check(f'oxc-{size}-honest-scope',all(f['status']=='local_facts_only' for f in oxc['files']))
    spans_valid=True
    for f in oxc['files']:
        source=fixture_source(f['file'])
        spans_valid &= all(source[n['start']:n['end']].decode()==n['text'] for n in f['nodes'])
    check(f'oxc-{size}-exact-byte-spans',spans_valid)
    for run in range(3):
        ts=read(results/f'{size}-typescript-{run:02}.stdout')
        check(f'ts6-{size}-{run}-alias-and-implementation',any(d['fileName'].endswith('/domain.ts') for d in ts['definitions']) and any(i['fileName'].endswith('/repository.ts') for i in ts['implementations']))
        check(f'ts6-{size}-{run}-twenty-fresh-dependent-types',len(ts['updates'])==20 and all(u['type']==('number' if u['edited'] else 'OrderId') and ((2345 in u['consumerDiagnostics'])==u['edited']) for u in ts['updates']))
        native=read(results/f'{size}-typescript7-lsp-final-{run:02}.json')
        check(f'ts7-lsp-{size}-{run}-real-alias-reference-implementation',all(native['queries'][i]['response'].get('result') for i in range(3)))
        check(f'ts7-lsp-{size}-{run}-twenty-fresh-dependent-types',len(native['updates'])==20 and all(f"const result: {u['expected']}" in u['query']['response']['result']['contents']['value'] for u in native['updates']))
        api=read(results/f'{size}-typescript7-api-final-{run:02}.stdout')
        check(f'ts7-api-{size}-{run}-typed-alias-and-span',api['initial']['type']=='OrderId' and not api['initial']['isError'] and api['alias']['name']=='normalize' and not api['alias']['isUnknown'] and api['alias']['declarations'][0]['end']>api['alias']['declarations'][0]['pos'])
        check(f'ts7-api-{size}-{run}-twenty-fresh-dependent-types',len(api['updates'])==20 and all(u['type']==u['expected'] for u in api['updates']))
        rust=read(results/f'{size}-rust-final-{run:02}.json')
        check(f'rust-{size}-{run}-healthy-ready',rust.get('ready',{}).get('health')=='ok' and 'error' not in rust)
        check(f'rust-{size}-{run}-navigation-facts',all(q['response'].get('result') for q in rust['queries'][:4]))
        macro=rust['queries'][5]['response']['result']
        check(f'rust-{size}-{run}-macro-text-only',set(macro)=={'name','expansion'} and 'fn generated()' in macro['expansion'])
        check(f'rust-{size}-{run}-cfg-invalidation',rust['cfg_hover']['response']['result'] is None and rust['feature_ready']['health']=='ok' and 'feature_only' in rust['feature_enabled_hover']['response']['result']['contents']['value'])
        check(f'rust-{size}-{run}-build-script-disabled',rust['build_script_executed'] is False)
        check(f'rust-{size}-{run}-twenty-fresh-dependent-layouts',len(rust['updates'])==20 and all(f"size = {4 if i%2==0 else 8}" in u['dependent']['response']['result']['contents']['value'] for i,u in enumerate(rust['updates'])))
    watch=read(results/f'{size}-mago-watch.json')
    check(f'mago-{size}-twenty-fresh-dependent-types','error' not in watch and len(watch['updates'])==20 and all(len(u['expression'])==1 and u['expression'][0]['type']==u['expected_type'] for u in watch['updates']))

oxc_errors=read(root/'oxc-smoke.json')['files']
check('oxc-invalid-syntax-visible',oxc_errors[1]['status']=='parse_failed')
check('oxc-missing-import-not-resolved-project-wide',oxc_errors[2]['status']=='local_facts_only' and not oxc_errors[2]['parse_errors'])
php=read(root/'mago-smoke.stdout')
check('mago-dynamic-dispatch-visible',any(i['code']=='string-member-selector' for i in php['issues']))
facts=json.loads((root/'fixtures-small-mago-facts.jsonl').read_text().splitlines()[0])
check('mago-interface-metadata',facts['class']['directParentInterfaces']==['fixture\\repository'])
check('mago-incoming-reference',any(e['source']['symbol']=='fixture\\serve' and e['target']=={'class':'fixture\\repository','member':'load'} for e in facts['incoming']))
check('mago-reference-edge-lacks-call-site',all(set(e)=={'source','target','kind'} for e in facts['incoming']))
without=read(root/'composer-without-includes.stdout');with_vendor=read(root/'composer-with-includes.stdout')
check('mago-composer-scope-must-include-vendor',any(i['code']=='non-existent-class-like' for i in without['issues']) and not any(i['code']=='non-existent-class-like' for i in with_vendor['issues']))
for name in ['php-parse','php-missing','php-capture-failure']:
    check(f'{name}-not-success',read(root/'checks'/(name+'.status.json'))['exit']!=0)
check('mago-capture-failure-explicit','Cannot retain evidence' in (root/'checks/php-capture-failure.stderr').read_text())
summary={'checks':checks,'passed':sum(c['passed'] for c in checks),'failed':[c['id'] for c in checks if not c['passed']]}
(root/'checks/qualification-checks.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps({k:summary[k] for k in ['passed','failed']}))
if summary['failed']:raise SystemExit(1)
