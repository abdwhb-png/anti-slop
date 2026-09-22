"""Small measurement harness; serialize benchmarks, retain stdout/stderr and process cost."""
from pathlib import Path
import json, os, signal, subprocess, threading, time

def process_tree_rss(pid):
    total=0; pending=[pid]
    while pending:
        current=pending.pop()
        try:
            status=Path(f'/proc/{current}/status').read_text()
            total+=int(next(l for l in status.splitlines() if l.startswith('VmRSS:')).split()[1])
            pending.extend(int(n) for n in Path(f'/proc/{current}/task/{current}/children').read_text().split())
        except (OSError,StopIteration):pass
    return total

def run(command,output,cwd,timeout=120):
    output=Path(output);output.parent.mkdir(parents=True,exist_ok=True)
    record={'command':[str(x) for x in command],'cwd':str(cwd),'load_before':os.getloadavg()}
    env={k:os.environ[k] for k in ['PATH','HOME','RUSTUP_HOME','LANG'] if k in os.environ}
    env.update({'NO_COLOR':'1','CARGO_NET_OFFLINE':'true','CARGO_HOME':str(Path(__file__).parent/'cargo-home')})
    timed=['/usr/bin/time','-f','%M','-o',str(output.with_suffix('.rss'))]+record['command']
    with output.with_suffix('.stdout').open('wb') as stdout, output.with_suffix('.stderr').open('wb') as stderr:
        started=time.perf_counter()
        process=subprocess.Popen(timed,cwd=cwd,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
        samples=[];expired=[]
        def sample():
            while process.poll() is None:
                samples.append(process_tree_rss(process.pid));time.sleep(.01)
        def cancel():
            expired.append(True)
            try:os.killpg(process.pid,signal.SIGKILL)
            except ProcessLookupError:pass
        thread=threading.Thread(target=sample,daemon=True);thread.start()
        timer=threading.Timer(timeout,cancel);timer.start()
        code=process.wait();elapsed=(time.perf_counter()-started)*1000
        timer.cancel();thread.join(timeout=1)
    rss=output.with_suffix('.rss').read_text().splitlines()
    record.update({'exit':code,'timeout':bool(expired),'wall_ms':elapsed,'max_process_rss_kib':int(rss[-1]) if rss and rss[-1].isdigit() else None,'sampled_tree_rss_kib':max(samples,default=0),'tree_samples':len(samples),'stdout_bytes':output.with_suffix('.stdout').stat().st_size})
    output.with_suffix('.measurement.json').write_text(json.dumps(record,indent=2)+'\n')
    return record
