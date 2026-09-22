"""Qualify Mago's existing experimental watch mode, including dependent reanalysis."""
from pathlib import Path
import json, os, signal, subprocess, sys, threading, time
from measure import process_tree_rss

def run(root,fixture,output,edits=20):
    facts=root/(fixture.name+'-mago-facts.jsonl');offset=facts.stat().st_size if facts.exists() else 0
    command=[str(root/'bin/mago'),'--workspace',str(fixture/'php'),'analyze','--watch','--reporting-format','json']
    file=fixture/'php/src/Repository.php';original=file.read_text();samples=[]
    with output.with_suffix('.stdout').open('wb') as stdout,output.with_suffix('.stderr').open('wb') as stderr:
        start=time.perf_counter()
        process=subprocess.Popen(command,stdout=stdout,stderr=stderr,start_new_session=True)
        def sample():
            while process.poll() is None:samples.append(process_tree_rss(process.pid));time.sleep(.02)
        threading.Thread(target=sample,daemon=True).start()
        def next_record(timeout=90):
            nonlocal offset
            deadline=time.monotonic()+timeout
            while time.monotonic()<deadline:
                if process.poll() is not None:raise RuntimeError(f'Mago watch exited {process.returncode}')
                if facts.exists():
                    with facts.open('rb') as stream:stream.seek(offset);line=stream.readline()
                    if line.endswith(b'\n'):
                        offset+=len(line);return json.loads(line)
                time.sleep(.005)
            raise TimeoutError('No fresh Mago evidence')
        report={'command':command,'updates':[]}
        try:
            first=next_record();report['initial_ms']=(time.perf_counter()-start)*1000
            report['initial_files']=len(first['files'])
            report['initial_tree_rss_kib']=process_tree_rss(process.pid)
            for index in range(edits):
                expected='int' if index%2==0 else 'string'
                start=time.perf_counter();file.write_text(original.replace('): string;',f'): {expected};'))
                record=next_record();elapsed=(time.perf_counter()-start)*1000
                service=next(f for f in record['files'] if f['file']=='src/Service.php')
                expression=[e for e in service['expressionTypes'] if service['source'][e['start']:e['end']]=='$repository->load($id)']
                report['updates'].append({'generation':index+1,'expected_type':expected,'expression':expression,'ms':elapsed,'files':len(record['files']),'issues':record['issueCount'],'tree_rss_kib':process_tree_rss(process.pid)})
            report['retained_tree_rss_kib']=process_tree_rss(process.pid)
        except Exception as error:report['error']=repr(error)
        finally:
            if file.read_text()!=original:file.write_text(original)
            os.killpg(process.pid,signal.SIGTERM)
            try:process.wait(timeout=5)
            except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait()
    report['sampled_peak_tree_rss_kib']=max(samples,default=0)
    output.write_text(json.dumps(report,indent=2)+'\n')
    return report

if __name__=='__main__':
    root=Path(__file__).resolve().parent
    report=run(root,Path(sys.argv[1]).resolve(),Path(sys.argv[2]).resolve())
    print(json.dumps({k:report.get(k) for k in ['initial_ms','initial_files','error','sampled_peak_tree_rss_kib','retained_tree_rss_kib']}))
    if 'error' in report:raise SystemExit(1)
