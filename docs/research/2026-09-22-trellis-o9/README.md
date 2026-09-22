# Trellis O9: focused CLI qualification

Date: 2026-09-22

Status: the native CLI is a technically viable candidate for a bounded TypeScript evidence adapter. Its incremental value over the future TypeScript analyzer is not established. O9 is not selected as a production dependency by this experiment.

## Decision this experiment supports

O9 can be evaluated before the full TypeScript/PHP/Rust qualification. This experiment qualifies the existing Trellis process boundary and identifies integration constraints. Final adoption requires comparison with the selected TypeScript evidence path on representative changes; it does not need to wait for PHP and Rust qualification.

## Scope and provenance

- Trellis source: [commit 42c2a26b7bbc6ff4fea08f838fff469af8c23fbe](https://github.com/jayminwest/trellis/tree/42c2a26b7bbc6ff4fea08f838fff469af8c23fbe); emitted analyzer version `0.3.0`, report schema `1.2.0`.
- Runtime: Bun `1.3.14`, Linux x86_64 / WSL2. The source CLI was invoked explicitly with Bun, automatic installation and `.env` loading disabled.
- Installation: production dependencies only, from the unchanged upstream `bun.lock`, through Socket Firewall `1.15.2`; lifecycle scripts disabled, seven-day minimum release age, and an isolated empty package cache. Registry versions and integrity values matched the lockfile for all five installed packages. See [registry-verification.json](registry-verification.json).
- No optional jscpd, dependency-cruiser, Knip or SonarJS provider was selected. No model was called. No Rust adapter or production component was implemented.
- Twelve synthetic mini-repositories were generated before running the detector. Their stated purposes are experiment expectations, not independent human adjudication or a representative accuracy corpus.
- The public Trellis checkout supplied the larger workload. No private user repository was benchmarked.

## Observations

| ID | Experiment | Observed result | O9 implication |
| --- | --- | --- | --- |
| Q1 | JSON process boundary, three runs per fixture | All 36 fixture audits emitted parseable versioned JSON. Measurements were equal after excluding execution metadata; fixture files were unchanged. | The normal report boundary is usable without parsing terminal presentation. |
| Q2 | Long duplicated billing rule, then centralized ownership | One clone group before, zero after; index 16 then 0. | Native clone evidence can select a useful candidate and observe a structural change. |
| Q3 | Similar calculations declared to belong to independent billing and sensor policies | One clone group and index 16, as for the shared billing rule. | A clone observation does not establish duplicated knowledge or justify refactoring. This is a limitation of interpreting the signal, not a claimed bug in clone detection. |
| Q4 | Repeated small order invariant; a chain of pass-through functions | Both produced zero findings and index 0. | Trellis cannot be the exclusive candidate generator for the agreed rule scope. These cases do not establish a population recall estimate. |
| Q5 | Runtime cycle and type-only cycle | Both found; metric details distinguished runtime from type-only. Both headline indices were 12. | Preserve edge semantics and evidence; do not substitute the aggregate index for our policy. |
| Q6 | Missing local import and malformed TypeScript | Both exited 0 under the neutral policy, while reports were explicitly incomplete/partial. Indices were 20 and 100 respectively. | Exit 0 is not analysis completeness, and a high partial index is not a training label for bad design. |
| Q7 | One TypeScript file plus one PHP and one Rust file | Report was complete for its native scope with index 0, and separately reported two unsupported files. | The core must reconcile declared language coverage; the child result is not a complete polyglot assessment. |
| Q8 | Comment shift and function rename | A comment shift preserved the named hotspot; rename produced a new/resolved pair. | Identity works for the tested edit class. Do not promise continuity across renames or extrapolate to every finding kind. |
| Q9 | Policy failure, missing root, unknown flag | Policy failure retained JSON on stdout with exit 2. Missing root returned JSON on stderr with exit 1. Unknown flag returned plain text on stderr with exit 1. | Normalize both streams and statuses in the adapter; the child CLI does not directly implement our AXI output contract. |
| Q10 | Selected-file fingerprints | All twelve fixtures' emitted native selection fingerprints matched their exact generated inputs. | The report can be bound to source content retained by our record store. Hashes alone do not preserve training context. |

The initial probe completed **44 checks with no failures**. A further **12 input-fingerprint checks passed**. These are focused experiment checks, not the upstream test suite or acceptance of every product requirement.

Read [summary.json](summary.json) for commands, repetitions, checks and measured case results. [probe.py](probe.py) is the reproducible initial harness. Raw stdout, stderr, RSS observations and generated fixture sources are retained in [evidence.tar.gz](evidence.tar.gz); [manifest.json](manifest.json) records artifact hashes.

## Measured execution cost

The initial probe disabled Bun's transpiler cache. It is retained as a controlled result, not used to claim ordinary warm-cache latency. A follow-up used an initially empty dedicated transpiler cache, then repeated fresh CLI processes with that cache warm. Filesystem caches were warm; host load was not isolated. Each row below represents three sequential invocations, with wall-clock startup and JSON rendering included.

| Workload | Median wall time | Largest peak RSS | Scope and interpretation |
| --- | --- | --- | --- |
| `--version`, dedicated cache population followed by two hits | 538 ms | 291 MiB, including the first cache-populating process | Later processes peaked near 130–131 MiB. This is a startup control, not an audit result. |
| One simple TypeScript file, cache warm | 596 ms | 137 MiB | Complete; approximately 13.6 KB JSON despite zero findings. |
| Trellis checkout with neutral configuration, cache warm | 4,088 ms | 470 MiB | Partial: included the intentional broken parsing fixture and unresolved dynamic imports. Not a complete-analysis speed claim. |
| Trellis checkout using its own `source.exclude: ["corpus/**"]`, cache warm | 9,708 ms | 461 MiB | 199 production files / 24,437 SLOC and 179 test files / 30,469 SLOC. Individual times: 11,305, 9,708 and 8,640 ms. Approximately 534 KB JSON. |

The configured checkout's remaining incomplete metrics concern two non-literal dynamic imports in `src/research/maintainability-spike.test.ts`. Their exact locations and reasons are retained in the reports. The native per-analysis states are `complete`, but affected metric states and the report's aggregate completeness still indicate the graph gap: an adapter must inspect the relevant metrics and coverage, not only a provider state.

The configured and unconfigured scopes differ, and host load was not controlled; the timings do not establish a causal performance comparison between configurations. No latency or memory acceptance budget has been agreed for our product. These measurements establish that the extra CLI audit has a material cost to include in the TypeScript comparison.

See [cache-probe.json](cache-probe.json) and [configured-probe.json](configured-probe.json) for exact commands, repetitions and conditions.

## Conditions for an adapter

1. **I1 — Own the contract.** Pin the executable and accepted report schema. Validate payloads, bound execution/output, and distinguish reports, policy failures, operational failures and malformed output. Normalize errors into our own public contract.
2. **I2 — Own selection and policy.** Record the effective source selection and exclusions. Keep useful source exclusions when preparing the child request; do not silently inherit a second CI blocking policy or optional-provider execution from the audited repository.
3. **I3 — Preserve uncertainty.** Read relevant metric states, report/evidence completeness, unsupported coverage and source identity. Advisory evidence may remain useful during a partial run; incompleteness must stay visible.
4. **I4 — Retain actual evidence.** Bind clone members, locations and measurements to our retained source snapshot. Do not treat a Trellis score as an adjudicated label or its report history as our training dataset.
5. **I5 — Demonstrate additional value.** Compare candidate coverage and false-positive burden against the qualified TypeScript analyzer, including small duplicated invariants and legitimate repeated structures. Measure the same post-agent changes with and without Trellis, including cold/warm behavior and output size.

The experiment supports proceeding with the TypeScript comparison. It does not yet justify a mandatory Trellis subprocess, an exclusive prefilter, or automatic CI blocking based on Trellis findings.

## Reproduction and remaining limits

Use the pinned source checkout and production dependencies above. The initial harness requires Python 3, Bun and GNU time, uses no Python dependencies, and refuses to overwrite an existing output directory:

```sh
python3 probe.py /path/to/pinned/trellis /path/to/new/results
```

The follow-up JSON artifacts retain their full command vectors. To reproduce the cache probe, use a dedicated `BUN_RUNTIME_TRANSPILER_CACHE_PATH` directory, keep `--no-install --no-env-file`, run `--version` three times, then audit the generated clean fixture three times. For the configured source measurement, audit the checkout with its own `trellis.yaml` three times using the same warm cache. Do not compare those runs to cache-disabled observations as if only the analyzed source had changed.

Not tested: the upstream full test suite, optional providers, hostile-process containment, cancellation/output-limit enforcement, a Rust adapter, model assessment accuracy, representative repository precision/recall, incremental native analysis, or added value over Oxc/another project-semantic source. The experiment changed no upstream tracked source or lockfile and introduced no production dependency into Anti-Slop.
