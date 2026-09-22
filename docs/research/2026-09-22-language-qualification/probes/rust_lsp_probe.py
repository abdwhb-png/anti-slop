"""Exercise actual rust-analyzer LSP, retaining raw messages and bounded timings."""
import json, os, queue, subprocess, sys, threading, time
from pathlib import Path
from measure import process_tree_rss

class Client:
    def __init__(self, binary, root, output, arguments=()):
        self.root=root.resolve();self.output=output;self.messages=[];self.serial=0;self.queue=queue.Queue();self.configuration={}
        self.stderr=open(output.with_suffix('.stderr'),'w')
        env=dict(os.environ,CARGO_NET_OFFLINE='true',CARGO_HOME=str(Path(__file__).resolve().parent/'cargo-home'))
        self.proc=subprocess.Popen([str(binary.resolve()),*arguments],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=self.stderr,cwd=root,env=env,start_new_session=True)
        self.peak_rss_kib=0;self.peak_tree_rss_kib=0
        self.reader=threading.Thread(target=self.read_loop,daemon=True);self.reader.start()
        self.sampler=threading.Thread(target=self.sample_loop,daemon=True);self.sampler.start()
    def sample_loop(self):
        while self.proc.poll() is None:
            try:
                rss=int(next(l for l in Path(f'/proc/{self.proc.pid}/status').read_text().splitlines() if l.startswith('VmRSS:')).split()[1])
                self.peak_rss_kib=max(self.peak_rss_kib,rss)
                self.peak_tree_rss_kib=max(self.peak_tree_rss_kib,process_tree_rss(self.proc.pid))
            except (OSError,StopIteration):pass
            time.sleep(.02)
    def read_loop(self):
        try:
            while True:
                headers={}
                while True:
                    line=self.proc.stdout.readline()
                    if not line: return
                    if line==b'\r\n':break
                    key,value=line.decode().split(':',1);headers[key.lower()]=value.strip()
                body=self.proc.stdout.read(int(headers['content-length']))
                self.queue.put(json.loads(body))
        finally:self.queue.put({'_eof':True})
    def send(self,message):
        self.messages.append({'direction':'send','message':message})
        body=json.dumps(message).encode()
        self.proc.stdin.write(f'Content-Length: {len(body)}\r\n\r\n'.encode()+body);self.proc.stdin.flush()
    def notification(self,method,params):self.send({'jsonrpc':'2.0','method':method,'params':params})
    def receive(self,deadline):
        message=self.queue.get(timeout=max(.001,deadline-time.monotonic()))
        self.messages.append({'direction':'receive','message':message})
        if message.get('_eof'):raise RuntimeError('LSP process closed')
        if 'id' in message and 'method' in message:
            result=[self.configuration] if message['method']=='workspace/configuration' else None
            self.send({'jsonrpc':'2.0','id':message['id'],'result':result})
        return message
    def request(self,method,params,timeout=90):
        self.serial+=1;serial=self.serial;t=time.perf_counter()
        self.send({'jsonrpc':'2.0','id':serial,'method':method,'params':params})
        deadline=time.monotonic()+timeout
        while True:
            message=self.receive(deadline)
            if message.get('id')==serial and 'method' not in message:
                return {'method':method,'ms':(time.perf_counter()-t)*1000,'response':message}
    def ready(self,timeout=90,require_busy=False):
        deadline=time.monotonic()+timeout
        busy=not require_busy
        while True:
            message=self.receive(deadline)
            if message.get('method')=='experimental/serverStatus':
                if not message['params'].get('quiescent'):busy=True
                elif busy:return message['params']
    def close(self):
        try:self.request('shutdown',None,timeout=5);self.notification('exit',None);self.proc.wait(timeout=5)
        except Exception:self.proc.kill();self.proc.wait()
        self.stderr.close()
        self.output.with_suffix('.messages.json').write_text(json.dumps(self.messages,indent=2)+'\n')

def position(text,needle,offset=1):
    index=text.index(needle)+offset
    before=text[:index]
    return {'line':before.count('\n'),'character':len(before.rsplit('\n',1)[-1].encode('utf-16-le'))//2}

def run(binary,root,output):
    started=time.perf_counter();client=Client(binary,root,output)
    report={'configuration':{'checkOnSave':False,'cargo':{'buildScripts':{'enable':False},'offline':True},'procMacro':{'enable':False},'cachePriming':{'enable':False},'numThreads':1},'queries':[]}
    client.configuration=report['configuration']
    try:
        report['initialize']=client.request('initialize',{'processId':os.getpid(),'rootUri':root.resolve().as_uri(),'capabilities':{'experimental':{'serverStatusNotification':True}},'initializationOptions':report['configuration']})
        client.notification('initialized',{})
        report['ready']=client.ready();report['ready_ms']=(time.perf_counter()-started)*1000
        domain=(root/'domain/src/lib.rs').resolve();consumer=(root/'app/src/lib.rs').resolve()
        domain_text=domain.read_text();consumer_text=consumer.read_text()
        client.notification('textDocument/didOpen',{'textDocument':{'uri':consumer.as_uri(),'languageId':'rust','version':1,'text':consumer_text}})
        client.notification('textDocument/didOpen',{'textDocument':{'uri':domain.as_uri(),'languageId':'rust','version':1,'text':domain_text}})
        cases=[
            ('textDocument/references',domain,domain_text,'Repository',{'context':{'includeDeclaration':True}}),
            ('textDocument/implementation',domain,domain_text,'Repository',{}),
            ('textDocument/definition',consumer,consumer_text,'load(&id)',{}),
            ('textDocument/typeDefinition',consumer,consumer_text,'id =',{}),
            ('textDocument/hover',consumer,consumer_text,'id =',{}),
            ('rust-analyzer/expandMacro',domain,domain_text,'make_getter!(generated)',{}),
        ]
        for method,file,text,needle,extra in cases:
            report['queries'].append(client.request(method,{'textDocument':{'uri':file.as_uri()},'position':position(text,needle),**extra}))
        report['facts_ready_ms']=(time.perf_counter()-started)*1000
        report['cfg_symbols']=client.request('workspace/symbol',{'query':'feature_only'})
        report['cfg_hover']=client.request('textDocument/hover',{'textDocument':{'uri':domain.as_uri()},'position':position(domain_text,'feature_only')})
        report['unchanged']=[]
        params={'textDocument':{'uri':consumer.as_uri()},'position':position(consumer_text,'id =')}
        for _ in range(20):report['unchanged'].append(client.request('textDocument/hover',params))
        report['updates']=[]
        for index in range(20):
            text=domain_text.replace('pub struct OrderId(pub u64);','pub struct OrderId(pub u32);') if index%2==0 else domain_text
            t=time.perf_counter()
            client.notification('textDocument/didChange',{'textDocument':{'uri':domain.as_uri(),'version':index+2},'contentChanges':[{'text':text}]})
            query=client.request('textDocument/hover',{'textDocument':{'uri':domain.as_uri()},'position':position(text,'pub u32' if index%2==0 else 'pub u64',4)})
            # Also query a dependent file after changing the shared type.
            dependent=client.request('textDocument/hover',params)
            report['updates'].append({'version':index+2,'ms':(time.perf_counter()-t)*1000,'query':query,'dependent':dependent})
        report['retained_rss_kib']=int(next(l for l in Path(f'/proc/{client.proc.pid}/status').read_text().splitlines() if l.startswith('VmRSS:')).split()[1])
        report['status']=client.request('rust-analyzer/analyzerStatus',{})
        while not client.queue.empty():client.receive(time.monotonic()+2)
        client.configuration=json.loads(json.dumps(report['configuration']))
        client.configuration['cargo']['features']=['qualification-domain/extra']
        client.notification('workspace/didChangeConfiguration',{'settings':{}})
        report['feature_ready']=client.ready(require_busy=True)
        report['feature_enabled_hover']=client.request('textDocument/hover',{'textDocument':{'uri':domain.as_uri()},'position':position(domain_text,'feature_only')})
        report['build_script_executed']=(root/'domain/qualification-build-script-ran').exists()
    except Exception as error:
        report['error']=repr(error)
    finally:
        report['sampled_peak_rss_kib']=client.peak_rss_kib
        report['sampled_peak_tree_rss_kib']=client.peak_tree_rss_kib
        client.close()
        output.write_text(json.dumps(report,indent=2)+'\n')
    return report

if __name__=='__main__':
    report=run(Path(sys.argv[1]),Path(sys.argv[2]),Path(sys.argv[3]))
    print(json.dumps({k:report.get(k) for k in ['ready_ms','ready','error','sampled_peak_rss_kib','retained_rss_kib']}))
    if 'error' in report:raise SystemExit(1)
