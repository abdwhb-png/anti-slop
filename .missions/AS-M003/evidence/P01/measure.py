"""Run the retained real-Git capture scenario and sample its whole process tree."""
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
CACHE = Path(os.environ.get("XDG_CACHE_HOME", str(Path.home() / ".cache"))) / "anti-slop" / hashlib.sha256(str(ROOT).encode()).hexdigest()[:16]
ENV = dict(os.environ, CARGO_TARGET_DIR=str(CACHE), PATH="/home/abdwhb/.cargo/bin:" + os.environ["PATH"])
build = subprocess.run(["cargo", "test", "--frozen", "--test", "workspace", "--no-run", "--message-format=json"], cwd=ROOT, env=ENV, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
(EVIDENCE / "measurement-build.log").write_text(build.stderr + "\nexit=" + str(build.returncode) + "\n")
build.check_returncode()
artifacts = [json.loads(line) for line in build.stdout.splitlines() if line.startswith("{")]
executable = next(item["executable"] for item in artifacts if item.get("executable") and item.get("target", {}).get("name") == "workspace")


def tree(pid):
    pending = [pid]
    seen = set()
    resident = 0
    observed = {}
    while pending:
        current = pending.pop()
        if current in seen:
            continue
        seen.add(current)
        try:
            status = Path(f"/proc/{current}/status").read_text()
            fields = dict(line.split(":", 1) for line in status.splitlines() if ":" in line)
            if int(fields["Tgid"]) != current:
                continue
            observed[current] = {"name": fields["Name"].strip(), "tgid": int(fields["Tgid"])}
            rss = next((int(line.split()[1]) for line in status.splitlines() if line.startswith("VmRSS:")), 0)
            resident += rss
            for children in Path(f"/proc/{current}/task").glob("*/children"):
                try:
                    pending.extend(int(n) for n in children.read_text().split())
                except (FileNotFoundError, ProcessLookupError):
                    pass
        except (FileNotFoundError, ProcessLookupError):
            pass
    return resident, observed


measurements = []
for count in (1, 5):
    output_path = EVIDENCE / f"observation-{count}.log"
    marker = EVIDENCE / f"observation-{count}.phase"
    marker.write_text("setup")
    with output_path.open("w") as output:
        command = [executable, "--ignored", "--exact", "capture_observations", "--nocapture"]
        start = time.perf_counter()
        process = subprocess.Popen(command, cwd=ROOT, env=dict(ENV, CAPTURE_OBSERVATION_FILES=str(count), CAPTURE_OBSERVATION_MARKER=str(marker)), stdout=output, stderr=subprocess.STDOUT)
        peak = 0
        capture_peak = 0
        capture_names = set()
        capture_max_processes = 0
        max_processes = 0
        seen = set()
        names = {}
        intervals = []
        last = time.perf_counter()
        while process.poll() is None:
            now = time.perf_counter()
            intervals.append((now - last) * 1000)
            last = now
            rss, current = tree(process.pid)
            peak = max(peak, rss)
            if marker.read_text() == "capture":
                capture_peak = max(capture_peak, rss)
                capture_names.update(v["name"] for v in current.values())
                capture_max_processes = max(capture_max_processes, len(current))
            max_processes = max(max_processes, len(current))
            seen |= current.keys()
            names.update(current)
            time.sleep(0.001)
        code = process.wait()
        duration = time.perf_counter() - start
    assert code == 0, output_path.read_text()
    observation = next(json.loads(line.partition("OBSERVATION ")[2]) for line in output_path.read_text().splitlines() if "OBSERVATION " in line)
    observation.update({"sampledPeakTreeRssKiBIncludingFixtureSetup": peak, "capturePhasePeakTreeRssKiB": capture_peak, "capturePhaseProcessNames": sorted(capture_names), "capturePhaseMaxObservedProcessesPerSweep": capture_max_processes, "maxObservedProcessesInNonAtomicSweep": max_processes, "observedProcessNames": sorted(set(v["name"] for v in names.values())), "observedThreadIdsMisclassified": sum(pid != v["tgid"] for pid, v in names.items()), "observedUniquePidsIncludingFixtureSetup": len(seen), "samplingIntervalP50Ms": statistics.median(intervals), "samplingIntervalMaxMs": max(intervals), "executableWallSecondsIncludingFixtureSetup": duration, "exit": code, "command": command})
    measurements.append(observation)

result = {"host": {"platform": platform.platform(), "machine": platform.machine(), "logicalCpus": os.cpu_count(), "loadAverage": os.getloadavg(), "rust": subprocess.check_output(["rustc", "--version"], env=ENV, text=True).strip(), "git": subprocess.check_output(["git", "--version"], text=True).strip()}, "sourceHead": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), "build": "debug dev profile, debug=1; persistent checkout-derived Cargo target; cached dependencies", "scenarios": measurements, "limitations": ["No previous capture implementation exists; these are scenario observations, not a speedup claim.", "RSS sums include the complete observed benchmark process tree and fixture setup. Process discovery and RSS reads are sequential, not atomic: a sweep can include sequential Git processes. Counts are observations per sweep, not proof of instantaneous concurrency. Sums may double count shared pages or sequential process readings, miss brief peaks, and are not a kernel-enforced memory bound.", "Every capture starts fresh Git processes; filesystem caches are warm from fixture creation; cold disk and reference repositories were not measured.", "Thirty warm samples and one first call per scenario do not establish reliable production tail latency.", "Capture only: no native analyzer, model, storage, serialization or resident lifecycle; ADR-007 budgets remain unqualified."]}
(EVIDENCE / "measurements.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps({"scenarios": [{k: v for k, v in s.items() if k not in ("warmCaptureMicros", "command")} for s in measurements]}, indent=2))
