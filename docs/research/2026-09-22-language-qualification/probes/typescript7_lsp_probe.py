"""Direct native TypeScript LSP; shares the real transport used by the Rust probe."""
from pathlib import Path
import json,os,sys,time
from rust_lsp_probe import Client,position

def run(binary,root,output):
    root=root.resolve();start=time.perf_counter();client=Client(binary,root,output,['--lsp','--stdio'])
    report={'analyzer':'typescript-native-lsp','version':'7.0.2','queries':[]}
    try:
        report['initialize']=client.request('initialize',{'processId':os.getpid(),'rootUri':root.as_uri(),'capabilities':{'textDocument':{'hover':{'contentFormat':['plaintext']}}}})
        client.notification('initialized',{})
        domain=root/'domain.ts';consumer=root/'service.ts';repository=root/'repository.ts'
        original=domain.read_text();consumer_text=consumer.read_text();repository_text=repository.read_text()
        for file in [domain,consumer,repository]:client.notification('textDocument/didOpen',{'textDocument':{'uri':file.as_uri(),'languageId':'typescript','version':1,'text':file.read_text()}})
        params={'textDocument':{'uri':consumer.as_uri()},'position':position(consumer_text,'result =')}
        report['first_hover']=client.request('textDocument/hover',params)
        report['first_result_ms']=(time.perf_counter()-start)*1000
        for method,file,text,needle,offset,extra in [
            ('textDocument/definition',consumer,consumer_text,'parseId("',1,{}),
            ('textDocument/references',domain,original,'normalize',1,{'context':{'includeDeclaration':True}}),
            ('textDocument/implementation',repository,repository_text,'interface Repository',len('interface ')+1,{}),
            ('textDocument/typeDefinition',consumer,consumer_text,'result =',1,{}),
        ]:report['queries'].append(client.request(method,{'textDocument':{'uri':file.as_uri()},'position':position(text,needle,offset),**extra}))
        report['facts_ready_ms']=(time.perf_counter()-start)*1000
        report['unchanged']=[client.request('textDocument/hover',params) for _ in range(20)]
        report['updates']=[]
        for i in range(20):
            edited=i%2==0;text=original.replace('): OrderId {','): number {').replace('return raw as OrderId;','return raw.length;') if edited else original
            started=time.perf_counter()
            client.notification('textDocument/didChange',{'textDocument':{'uri':domain.as_uri(),'version':i+2},'contentChanges':[{'text':text}]})
            query=client.request('textDocument/hover',params)
            report['updates'].append({'version':i+2,'expected':'number' if edited else 'OrderId','ms':(time.perf_counter()-started)*1000,'query':query})
        report['retained_rss_kib']=int(next(l for l in Path(f'/proc/{client.proc.pid}/status').read_text().splitlines() if l.startswith('VmRSS:')).split()[1])
    except Exception as error:report['error']=repr(error)
    finally:
        report['sampled_peak_rss_kib']=client.peak_rss_kib;report['sampled_peak_tree_rss_kib']=client.peak_tree_rss_kib;client.close()
        output.write_text(json.dumps(report,indent=2)+'\n')
    return report

if __name__=='__main__':
    report=run(Path(sys.argv[1]),Path(sys.argv[2]),Path(sys.argv[3]));print(json.dumps({k:report.get(k) for k in ['first_result_ms','facts_ready_ms','retained_rss_kib','error']}))
    if 'error' in report:raise SystemExit(1)
