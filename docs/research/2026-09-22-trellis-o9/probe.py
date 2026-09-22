#!/usr/bin/env python3
"""Bounded black-box O9 probe; not a representative accuracy benchmark."""
import argparse
import collections
import copy
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import time

PIN = "42c2a26b7bbc6ff4fea08f838fff469af8c23fbe"


def write(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def digest_tree(root):
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(root.rglob("*")) if p.is_file()}


def measured_body(report):
    body = copy.deepcopy(report)
    body.pop("run", None)
    for analysis in body.get("evidence", {}).get("analyses", []):
        analysis.pop("execution", None)
    return body


def metric(report, name):
    return report.get("metrics", {}).get(name, {}).get("value")


def long_rule(name, cap=50000):
    return f'''export function {name}(values: number[], adjustment: number): number {{
  let sum = 0;
  for (const value of values) {{
    const normalized = Number.isFinite(value) ? value : 0;
    const bounded = Math.max(-1000, Math.min(1000, normalized));
    const weighted = bounded * 1.25;
    const corrected = weighted + adjustment;
    const rounded = Math.round(corrected * 100) / 100;
    sum += rounded;
  }}
  const count = values.length;
  const average = count === 0 ? 0 : sum / count;
  const capped = Math.min({cap}, Math.max(0, average));
  return Math.round(capped * 100) / 100;
}}
'''


def fixtures():
    hotspot = "export function classify(value: number): number {\n"
    hotspot += "".join(f"  if (value === {n}) return {n * 3};\n" for n in range(16))
    hotspot += "  return -1;\n}\n"
    return {
        "clean": ("Simple complete negative control", {"src/main.ts": "export const twice = (x: number) => x * 2;\n"}),
        "shared-rule-duplicate": ("Two owners of the same declared billing calculation", {
            "src/web-billing.ts": long_rule("webBill"), "src/batch-billing.ts": long_rule("batchBill")}),
        "shared-rule-centralized": ("Same billing calculation now owned once", {
            "src/billing.ts": long_rule("calculateBill"),
            "src/web-billing.ts": "export { calculateBill as webBill } from './billing';\n",
            "src/batch-billing.ts": "export { calculateBill as batchBill } from './billing';\n"}),
        "independent-similarity": ("Intentionally independent domain policies; similar syntax is not by itself a design defect", {
            "src/billing.ts": "// Billing owns this calculation and changes independently of sensor policy.\n" + long_rule("bill", 50000),
            "src/sensor.ts": "// Sensor calibration owns this calculation and changes independently of billing.\n" + long_rule("sensorReading", 20000)}),
        "short-shared-rule": ("Repeated ownership of the same small order invariant, below native clone minima", {
            "src/web.ts": "export const webCanOrder = (total: number) => total >= 75 && total <= 5000;\n",
            "src/batch.ts": "export const batchCanOrder = (total: number) => total >= 75 && total <= 5000;\n"}),
        "pass-through": ("Three delegations without additional behavior; qualitative candidate outside native metric catalog", {
            "src/main.ts": "const amount = (x: number) => x * 2;\nconst service = (x: number) => amount(x);\nconst facade = (x: number) => service(x);\nexport const endpoint = (x: number) => facade(x);\n"}),
        "runtime-cycle": ("Two modules forming a runtime import cycle", {
            "src/a.ts": "import { b } from './b';\nexport function a(x: number): number { return x ? b(x - 1) : 0; }\n",
            "src/b.ts": "import { a } from './a';\nexport function b(x: number): number { return x ? a(x - 1) : 0; }\n"}),
        "type-cycle": ("Type-only recursive relationship, distinct from a runtime cycle", {
            "src/a.ts": "import type { B } from './b';\nexport interface A { peer?: B; }\n",
            "src/b.ts": "import type { A } from './a';\nexport interface B { peer?: A; }\n"}),
        "unresolved-import": ("Missing local module must remain visible as incomplete analysis", {
            "src/main.ts": "import { missing } from './absent';\nexport const result = missing();\n"}),
        "parse-error": ("Malformed TypeScript must not become a clean complete result", {
            "src/main.ts": "export function broken( { return 42;\n"}),
        "mixed-languages": ("PHP and Rust files must appear as unsupported coverage", {
            "src/main.ts": "export const value = 1;\n", "src/legacy.php": "<?php function value() { return 1; }\n",
            "src/lib.rs": "pub fn value() -> u8 { 1 }\n"}),
        "hotspot": ("Named complex function for baseline attribution checks", {"src/main.ts": hotspot}),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("checkout", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    repo, out = args.checkout.resolve(), args.output.resolve()
    if out.exists():
        raise SystemExit("Use a new output directory; existing evidence will not be overwritten")
    sha = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip()
    if sha != PIN:
        raise SystemExit("Checkout does not match the qualified commit")
    out.mkdir(parents=True)
    bun = shutil.which("bun")
    if not bun or not Path("/usr/bin/time").exists():
        raise SystemExit("Bun and GNU time are required")
    env = {"PATH": os.environ["PATH"], "LANG": "C.UTF-8", "NO_COLOR": "1",
           "BUN_RUNTIME_TRANSPILER_CACHE_PATH": "0"}
    neutral = out / "neutral.yaml"
    write(neutral, "{}\n")
    runs, results, checks = [], {}, []

    def invoke(name, command):
        output_file = out / "runs" / f"{name}.stdout"
        error_file = out / "runs" / f"{name}.stderr"
        rss_file = out / "runs" / f"{name}.rss"
        output_file.parent.mkdir(exist_ok=True)
        argv = [bun, "--no-install", "--no-env-file", str(repo / "src/cli/main.ts"), *command]
        started = time.monotonic()
        proc = subprocess.run(["/usr/bin/time", "-q", "-f", "%M", "-o", str(rss_file), *argv],
                              cwd=repo, env=env, capture_output=True, timeout=45)
        elapsed = (time.monotonic() - started) * 1000
        output_file.write_bytes(proc.stdout)
        error_file.write_bytes(proc.stderr)
        try:
            payload = json.loads(proc.stdout)
        except (ValueError, UnicodeDecodeError):
            payload = None
        rss = rss_file.read_text().strip()
        row = {"name": name, "command": argv, "exit": proc.returncode, "wall_ms": round(elapsed, 2),
               "peak_rss_kib": int(rss) if rss.isdigit() else None,
               "stdout_bytes": len(proc.stdout), "stderr_bytes": len(proc.stderr),
               "stdout_json": payload is not None, "stdout_file": str(output_file.relative_to(out)),
               "stderr_file": str(error_file.relative_to(out))}
        runs.append(row)
        print(json.dumps({k: row[k] for k in ("name", "exit", "wall_ms", "stdout_bytes", "stdout_json")}), flush=True)
        return payload, row

    def audit(name, path, extra=()):
        return invoke(name, ["audit", str(path), "--config", str(neutral), "--json", "--quiet", *extra])

    def check(name, passed, observed):
        checks.append({"name": name, "passed": bool(passed), "observed": observed})

    for name, (purpose, files) in fixtures().items():
        root = out / "fixtures" / name
        write(root / "package.json", json.dumps({"name": name, "private": True}) + "\n")
        for relative, content in files.items():
            write(root / relative, content)
        before = digest_tree(root)
        measured = [audit(f"{name}-{i}", root) for i in range(3)]
        reports = [item[0] for item in measured]
        report = reports[0]
        check(f"{name}: valid JSON reports", all(isinstance(r, dict) and "schemaVersion" in r for r in reports),
              [item[1]["exit"] for item in measured])
        check(f"{name}: no target writes", digest_tree(root) == before, before)
        check(f"{name}: deterministic measurement", all(measured_body(r) == measured_body(report) for r in reports),
              "run and per-analysis execution metadata excluded")
        results[name] = {"purpose": purpose, "index": report["score"]["index"],
                         "completeness": report["completeness"], "partial": report["score"]["partial"],
                         "source_coverage": report["sourceCoverage"],
                         "finding_kinds": dict(collections.Counter(f["kind"] for f in report["findings"])),
                         "clone_groups": metric(report, "duplication.groups.production"),
                         "cycle_groups": metric(report, "import-cycle.groups"),
                         "median_wall_ms": round(statistics.median(item[1]["wall_ms"] for item in measured), 2),
                         "max_peak_rss_kib": max(item[1]["peak_rss_kib"] for item in measured),
                         "reports": [item[1]["stdout_file"] for item in measured]}

    check("long shared rule generates a clone candidate", results["shared-rule-duplicate"]["clone_groups"] > 0,
          results["shared-rule-duplicate"]["clone_groups"])
    check("centralizing the rule removes clone candidates", results["shared-rule-centralized"]["clone_groups"] == 0,
          results["shared-rule-centralized"]["clone_groups"])
    for name in ("unresolved-import", "parse-error"):
        check(f"{name}: explicitly incomplete", results[name]["completeness"] == "incomplete", results[name]["completeness"])

    hotspot_root = out / "fixtures" / "hotspot"
    baseline = out / results["hotspot"]["reports"][0]
    original = (hotspot_root / "src/main.ts").read_text()
    write(hotspot_root / "src/main.ts", "// Shift source locations only.\n\n" + original)
    audit("hotspot-shift", hotspot_root)
    shifted, _ = invoke("compare-shift", ["compare", str(baseline), str(out / "runs/hotspot-shift.stdout"), "--json"])
    check("comment shift preserves named hotspot identity",
          len(shifted.get("findings", {}).get("new", [])) == 0 and len(shifted.get("findings", {}).get("persistent", [])) > 0,
          shifted.get("findings"))
    write(hotspot_root / "src/main.ts", original.replace("classify", "classifyRenamed"))
    audit("hotspot-rename", hotspot_root)
    renamed, _ = invoke("compare-rename", ["compare", str(baseline), str(out / "runs/hotspot-rename.stdout"), "--json"])
    results["baseline_comparisons"] = {"comment_shift": shifted.get("findings"), "rename": renamed.get("findings")}

    policy = out / "policy.yaml"
    write(policy, "policy:\n  maxIndex: 0\n")
    report, row = invoke("policy-failure", ["audit", str(out / "fixtures/shared-rule-duplicate"), "--config", str(policy), "--json", "--quiet"])
    check("policy failure retains JSON report with exit 2", row["exit"] == 2 and report is not None and "schemaVersion" in report, row)
    for name, command in (
        ("missing-root", ["audit", str(out / "does-not-exist"), "--json", "--quiet"]),
        ("unknown-flag", ["audit", str(out / "fixtures/clean"), "--json", "--unknown-o9-flag"]),
    ):
        _, row = invoke(name, command)
        check(f"{name}: nonzero failure", row["exit"] != 0, row)

    self_runs = [audit(f"trellis-source-{i}", repo) for i in range(3)]
    report = self_runs[0][0]
    results["trellis-source"] = {"purpose": "Public real source tree; not a representative user workload",
                                "source_coverage": report["sourceCoverage"], "completeness": report["completeness"],
                                "finding_count": len(report["findings"]),
                                "median_wall_ms": round(statistics.median(row["wall_ms"] for _, row in self_runs), 2),
                                "max_peak_rss_kib": max(row["peak_rss_kib"] for _, row in self_runs),
                                "max_stdout_bytes": max(row["stdout_bytes"] for _, row in self_runs)}
    metadata = {"commit": PIN, "bun": subprocess.check_output([bun, "--version"], text=True).strip(),
                "platform": platform.platform(), "measurement": "Three sequential fresh processes; warm filesystem cache; Bun transpiler cache disabled. No cold-cache claim.",
                "scope": "Native Trellis CLI only; optional providers, Rust adapter and comparison with alternative TS analyzers are not qualified."}
    summary = {"metadata": metadata, "cases": results, "checks": checks, "runs": runs}
    write(out / "summary.json", json.dumps(summary, indent=2) + "\n")
    print(json.dumps({"summary": str(out / "summary.json"), "checks": len(checks), "failed": [c["name"] for c in checks if not c["passed"]]}), flush=True)
    return 1 if any(not c["passed"] for c in checks) else 0


if __name__ == "__main__":
    raise SystemExit(main())
