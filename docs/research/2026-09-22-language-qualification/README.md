# Initial language evidence qualification

Date: 2026-09-22. Scope: reversible experiments supporting D2, D4 and D8 in the [design](../../brainstorming-decisions/2026-09-22-anti-slop-design.md). O9 remains deferred. These are analyzer probes, not an Anti-Slop implementation or acceptance benchmark.

## Findings and recommendation

| ID | Observed result | Design consequence |
| --- | --- | --- |
| Q1 | Oxc exposes syntax, comments, byte spans, symbols and local references cheaply. TypeScript 7's native server resolves project aliases, references and implementations; its separate typed API also exposes the inferred domain type and declaration spans. | Keep Oxc and native TypeScript as complementary candidates. A Rust LSP client can avoid a Node process for navigation, but LSP hover text is not the rich typed contract needed by every rule. Qualify that boundary before selecting the production adapter. |
| Q2 | Mago's real PHP SDK extracts expression types, source text, class/interface metadata and symbol-reference edges. Dependent types update after an interface edit. Composer dependencies require inclusion in Mago's source scope. | Mago is a viable evidence candidate. Preserve dependency scope and dynamic-resolution limits. A PHP worker has demonstrated value; a Rust worker or embedded Mago integration remains untested. |
| Q3 | rust-analyzer resolves the controlled workspace, trait implementation, types, cfg changes and a declarative macro. It retains approximately 476–487 MiB and takes several seconds to produce the first queried facts with the normal sysroot enabled. | Capability is demonstrated; resource acceptance is not. Do not select an always-resident Rust backend or assume embedding its libraries solves the cost. Resolve the resource trade-off before adopting this integration for repeated Pi use. |
| Q4 | All 97 focused checks pass, including dependent updates, explicit failure cases and disabled build-script execution. | These validate particular extraction behaviors. They do not establish R1–R9 accuracy, blocking eligibility, a complete dataset pipeline or an end-to-end performance budget. |

This narrows the integration choices without changing D1–D8. No persistent service, analyzer dependency or model provider is approved for production by this report. PHPStan was not needed to establish initial PHP extraction feasibility and was not benchmarked.

## Versions and execution conditions

| Component | Tested version and source |
| --- | --- |
| Oxc | Exact Rust crates 0.150.0; [source commit](https://github.com/oxc-project/oxc/tree/5a6e37e5cf895143a5b34050c50109c46e2ae96a); release-mode probe, Rust 1.96.0. |
| TypeScript native | 7.0.2, existing installed native Linux binary and JavaScript API client; [registry metadata](https://registry.npmjs.org/typescript/7.0.2). The package exposes the typed clients as `unstable/sync` and `unstable/async`. |
| TypeScript reference | 6.0.3 JavaScript LanguageService, reused from an existing temporary dependency tree; [registry metadata](https://registry.npmjs.org/typescript/6.0.3). This is not a new Trellis run. |
| Mago | Official [1.48.1 release](https://github.com/carthage-software/mago/releases/tag/1.48.1), matching [SDK source](https://github.com/carthage-software/mago/tree/4b833d559b9fc21da7c49a2458d0d057d9b2227a/composer), PHP 8.5.0 and Revolt event-loop 1.0.9. |
| rust-analyzer | Official [2026-09-14 release](https://github.com/rust-lang/rust-analyzer/releases/tag/2026-09-14), source `682a84e95b5a52cf06e9822fcaa4f628738554bf`, installed Rust 1.96.0 sysroot. |

The host is WSL2 x86-64. Exact versions, selected executable/client hashes, source commits and release archive digests are in [provenance.json](provenance.json). Existing TypeScript files were identified and hashed; they were not freshly installed or independently verified against an entire package tarball during this qualification.

Cargo dependencies were fetched through Socket Firewall into an isolated temporary cache. The Oxc lockfile contains 83 registry packages with checked registry checksums and yank status; selected versions were at least seven days old at qualification. The installed rust-src workspace required 40 locked registry dependencies, also checked and fetched through Socket Firewall. `RUSTC_BOOTSTRAP=1` was required only to read the sysroot Cargo manifest's unstable profile settings during fetch. No toolchain was changed; the installed sysroot lockfile stayed unchanged.

The user explicitly authorized official Mago and rust-analyzer release downloads outside Socket Firewall, with publisher SHA-256 verification. The user separately authorized the SDK's sole Composer registry dependency, `revolt/event-loop` 1.0.9, in the temporary directory. Composer ran without scripts/plugins, wrote a lockfile, and reported no advisories or abandoned packages. A separate controlled Composer path package was installed offline to test dependency resolution. These exceptions do not authorize unrestricted dependency installation in the product.

## Fixtures and coverage

| Fixture | TypeScript sources | PHP sources | Rust sources |
| --- | --- | --- | --- |
| Small | 3 files; 702 bytes; 16 lines | 4 files; 761 bytes; 25 lines | 3 files; 728 bytes; 14 lines |
| Medium | 203 files; 293,182 bytes; 9,816 lines | 204 files; 157,251 bytes; 3,225 lines | 203 files; 108,018 bytes; 2,614 lines |

The medium fixtures add 200 generated files per language with twelve functions each. Their shapes and analyzer workloads differ; file count does not make them equivalent benchmark projects. Counts exclude configuration and dependency source. The Rust count includes the build-script marker fixture. Source files and configurations are fingerprinted in [fixture-manifest.json](fixture-manifest.json).

The TypeScript fixture combines a branded `OrderId`, a path alias, an aliased import, an interface with a useful implementation, boundary validation and a dynamic import. The PHP fixture supplies similar domain/interface relationships, repeated validation and unresolved dynamic dispatch. A second PHP project imports `Catalog\ExternalId` from a local Composer path dependency. The Rust workspace contains two crates, a trait and implementation, an inactive feature, a declarative macro and a build script that would leave a marker if executed.

### TypeScript observations

Oxc's probe returns real syntax and local semantic facts, with source text checked against byte spans. Its output declares `local_facts_only`. A nonexistent import is still locally bound: successful parsing and local binding do not establish project resolution. Invalid syntax is explicitly reported as `parse_failed`.

The TypeScript 7 native binary runs directly with `--lsp --stdio`. The experiment observes aliased definitions, references, implementations and an inferred `OrderId` hover. Twenty alternating changes to the domain function's return type per session propagate to the consumer's inferred type. However, `textDocument/typeDefinition` at that consumer variable returns `null` for this branded type even when hover reports `OrderId`. The output must not invent a type-definition target.

The actual `unstable/sync` API supplies the type object, flags, error-type status, an aliased symbol and declaration offsets via resolved node handles. Its snapshot update also propagates twenty real source-file edits per session. This is a version-pinned JavaScript client talking to the native process, not an embedded Rust type API. No production adapter was built, and parsing hover prose is not qualified as a replacement for structured types.

The TypeScript 6 reference probe additionally runs project diagnostics and confirms the expected consumer error when the shared type changes. Its greater scope and different timing boundary prevent treating it as a direct speed comparison with the TypeScript 7 LSP queries.

### PHP observations

The worker imports the pinned Mago SDK and uses `AfterAnalysisHook`, native expression types, source objects, metadata and reference queries. It retains real source text and expression spans, not just diagnostic messages. An edge from `Fixture\serve` to `Fixture\Repository::load` is present; the returned edge has `source`, `target` and `kind`, but no exact call-site span. Per-file expression spans are a separate source of location evidence.

The reference summary includes built-in stub relationships. Its totals are not a count of project complexity. The dynamic method selector remains an explicit `string-member-selector` issue. Invalid syntax and missing classes return nonzero statuses. An unwritable evidence destination causes an explicit worker failure and Mago exit 2; it is not reported as successful data capture.

With only `paths = ["src"]`, the Composer dependency's class is unresolved. Adding `includes = ["vendor"]` resolves the class and its `int` property. The fixture proves this scope requirement; it does not establish Laravel/Symfony container, facade, magic-member or other framework coverage.

The existing experimental watch mode was exercised separately. Each of twenty interface return-type changes produces a new evidence record with the corresponding dependent call type. Its measured delay includes filesystem observation/coalescing, analysis, full evidence extraction and append. It is not an isolated measure of incremental analysis time.

### Rust observations

The real LSP client waits for healthy, quiescent `experimental/serverStatus` before querying facts. The final runs use the normal sysroot, `numThreads = 1`, and disable cache priming, build scripts, procedural macros and check-on-save. An earlier missing-sysroot-dependency warning is retained as a rejected smoke run, not included in the final measurements.

Definitions, references, implementation locations, type-definition locations, hover and declarative macro expansion are available on the fixture. Twenty shared-field type changes per session update the dependent value's reported layout from eight to four bytes and back. Enabling the feature through `workspace/configuration` changes the previously absent hover to the feature function after project reload. The controlled build-script marker remains absent.

The macro extension returns `{name, expansion}` text, without a token-by-token mapping to original source. Standard LSP navigation supplies locations, not a complete reference graph with edge categories, provenance and completeness. A hover type or type-definition location is not a structured type graph. Build-generated code and procedural macro expansion remain outside the configured coverage.

A completed Terra research report independently examined Rust interface boundaries. That source review used revision `7ac8fbfeecbf8030eb9f9058313c21c933bcdf1c`, not the tested release; its broader API observations are not substituted for release-specific runtime evidence. See [LSP extensions](https://github.com/rust-lang/rust-analyzer/blob/7ac8fbfeecbf8030eb9f9058313c21c933bcdf1c/docs/book/src/contributing/lsp-extensions.md) and [IDE boundary](https://github.com/rust-lang/rust-analyzer/blob/7ac8fbfeecbf8030eb9f9058313c21c933bcdf1c/crates/ide/src/lib.rs). Richer embedded HIR/IDE access still requires a separate compatibility and resource experiment.

## Measurements

The authoritative observations are in [evidence.tar.gz](evidence.tar.gz), with recomputable statistics in [performance-summary.json](performance-summary.json). The following values are rounded. All timings are milliseconds; memory is MiB (1,048,576 bytes).

### Full one-shot work

Each cell below reports small / medium. There are ten fresh process runs per size, with warmed filesystem caches.

| Work performed | Wall p50 | Wall p95 | Largest process RSS | Sampled peak sum of process-tree RSS |
| --- | --- | --- | --- | --- |
| Oxc: selected-file local facts and JSON output | 3.4 / 105.1 | 6.0 / 136.8 | 3.7 / 33.6 | 2.7 / 35.1 |
| Mago: native analysis and diagnostics, no worker | 85.8 / 124.7 | 111.4 / 216.9 | 95.6 / 105.9 | 97.2 / 107.4 |
| Mago: analysis plus PHP SDK evidence capture | 151.0 / 332.3 | 174.0 / 459.7 | 105.9 / 116.3 | 138.3 / 161.3 |

Oxc's medium JSON output is approximately 1.61 MB. Mago's stdout contains diagnostics; SDK evidence is appended to a separate JSONL file. Thus the `output_bytes` field in a Mago process measurement does not represent its evidence volume. These broad probes deliberately retain more than a future targeted rule request may need.

### Loaded native semantic servers

There are three fresh server sessions per size. Each session performs twenty unchanged queries and twenty edits; repeated-query statistics combine sixty observations per size.

| Server and fixture | First queried facts p50 | Unchanged query p50 / p95 | Dependent update p50 / p95 | Sampled peak tree RSS | Largest final server RSS |
| --- | --- | --- | --- | --- | --- |
| TypeScript 7 LSP, small | 206.7 | 0.76 / 1.91 | 6.04 / 14.06 | 57.0 | 57.1 |
| TypeScript 7 LSP, medium | 225.1 | 0.58 / 0.76 | 3.77 / 10.12 | 89.3 | 89.7 |
| rust-analyzer, small | 8,368.2 | 0.83 / 1.81 | 31.74 / 139.41 | 490.3 | 476.0 |
| rust-analyzer, medium | 5,887.9 | 0.56 / 0.98 | 20.67 / 64.44 | 501.9 | 487.4 |

“First queried facts” includes process launch, initialization and the initial query sequence; it does not mean the entire project graph has been exported. TypeScript updates query one consumer hover; Rust updates query both the changed field and a dependent hover. Neither runs a full anti-slop rule assessment. Rust startup ranges are 5.93–8.82 seconds for small and 5.47–6.65 seconds for medium. The reversed medians illustrate host/run variability rather than a benefit from adding source files.

### Additional interfaces

| Interface | Initial operation p50, small / medium | Dependent update p50 / p95, small | Dependent update p50 / p95, medium | Memory, small / medium |
| --- | --- | --- | --- | --- |
| TypeScript 7 typed API | 164.3 / 237.2 | 7.66 / 11.72 | 9.39 / 18.15 | 115.7 / 130.2 peak sampled tree, including Node client |
| TypeScript 6 LanguageService | 654.0 / 803.6 | 19.84 / 50.13 | 19.75 / 43.41 | 291.8 / 411.1 largest final Node RSS |
| Mago experimental watch | 1,532.5 / 1,072.6 (one session each) | 324.5 / 334.9 | 496.8 / 529.2 | 174.0 / 196.2 retained tree; 200.5 / 220.5 sampled peak tree |

The TypeScript API initial timers start after loading the JavaScript module; TypeScript 7's timer also excludes constructing the client. They measure the first snapshot/project type operation, not complete process startup. TypeScript 6's complete initial navigation-plus-diagnostics stage takes 746.4 / 1,521.1 ms median. Mago watch updates have twenty observations per size and include delivery of a complete evidence record. Keeping that watch process resident is not automatically faster than these one-shot SDK runs.

### Measurement limits

- **QL1 — Workload:** generated projects establish specific capabilities, not representative monorepo, framework or dependency-heavy behavior. These workloads differ from the older Trellis experiment; no cross-experiment speedup is claimed.
- **QL2 — Host and sample size:** runs used a live WSL host and warm filesystem/page caches. Analyzer batches were serialized, but a short Oxc formatting rebuild overlapped part of the final Rust sequence. Startup has only three observations per server/size; p95 uses nearest rank and is simply the maximum for n=3. Warm observations are correlated within a few sessions. These are exploratory measurements, not isolated latency tails or cold-storage benchmarks.
- **QL3 — Memory:** GNU `time` reports the largest process high-water RSS, not simultaneous tree memory. Tree RSS is sampled every 10 ms for one-shot runs and 20 ms for LSP/watch runs; shared pages may be counted more than once and brief peaks may be missed. This explains sampled values below an independently observed final/high-water RSS. Retained values are immediate post-work snapshots, not a long-idle retention study.
- **QL4 — Product scope:** no Rust orchestration adapter, candidate selection, rule assessment, model inference, durable telemetry journal, feedback/adjudication, dataset export, TOON codec or Pi injection was measured. Cancellation, eviction, multiple concurrent workspaces and long-session resource bounds remain unqualified.

The measured evidence capture shows that native facts can be retained. It is not a complete training dataset: no findings were judged or labeled, and no user feedback was converted into ground truth. The provider-neutral record and model replacement requirements remain intact.

## Evidence replay and reruns

The archive contains generated fixtures, raw stdout/stderr, JSON-RPC traffic, timing records, Mago JSONL evidence and failure checks. It excludes analyzer executables, Cargo build output and downloaded registry packages. The Composer fixture includes its generated autoloader and the locally authored path package. Earlier smoke runs remain visibly separate from `*-final-*` observations.

To validate the retained observations without installing or executing an analyzer, run from this report directory:

```sh
qualification_replay=$(mktemp -d /tmp/anti-slop-evidence-replay.XXXXXX)
tar -xzf evidence.tar.gz -C "$qualification_replay"
python3 probes/verify_results.py "$qualification_replay"
python3 probes/summarize_results.py "$qualification_replay" > "$qualification_replay/summary.json"
cmp performance-summary.json "$qualification_replay/summary.json"
```

The verifier remaps recorded absolute source paths into the extracted fixture tree. It does not need the original temporary workspace. `artifact-manifest.json` fingerprints the retained report files, and `fixture-manifest.json` fingerprints archived fixture content.

For new live measurements, copy `probes/` into a fresh temporary work directory. Provision the exact verified binaries as `bin/mago` and `bin/rust-analyzer`, matching Mago source as `mago-source/`, and the locked SDK Composer dependency as `php-sdk/vendor/`. Use the checked-in Oxc Cargo lockfile with a release build. Dependency installation remains subject to the applicable supply-chain policy; the replay procedure above needs none.

The fixture generator creates the small workload with count `0` and medium with count `200`. Configure each PHP fixture's `extension-hosts.qualification` with one worker and a command containing the PHP executable, `mago_worker.php`, `mago-source/composer/src`, `php-sdk/vendor/autoload.php` and an absolute `<fixture-name>-mago-facts.jsonl` output path. Retained `mago.toml` files show the exact tested command. The archived Composer fixture is the source of truth for that additional scope check.

`run_measurements.py <new-output-directory> <typescript6-lib/typescript.js>` runs Oxc, both Mago modes and the reference TS6 probe. Its Oxc results use `oxc-*`; the archived `oxc-final-*` names identify the rerun after the final Rust probe formatting/build. Native interfaces are separately invoked as follows, using fresh output paths and the same three-session/twenty-edit schedule:

```text
python3 rust_lsp_probe.py <ra-binary> <fixture/rust> <output.json>
python3 typescript7_lsp_probe.py <native-tsc> <fixture/ts> <output.json>
node typescript7_api_probe.mjs <api/sync/api.js> <native-tsc> <fixture/ts>
python3 mago_watch_probe.py <fixture> <output.json>
```

Use `measure.run` around the Node typed-API command to capture whole-process/tree cost. Preserve source restoration and offline sysroot dependencies. Raw process command vectors and configuration are retained beside the observations. These probes provide bounded experiments, not a general benchmark runner or a production process manager.

## Remaining qualification boundary

The initial extraction experiments are complete. The main unresolved acceptance issue is Rust's resource footprint; numeric cold/warm/retained-memory budgets must be chosen before an integration is accepted. Candidate interfaces also need evidence contracts that expose missing coverage, without turning absent locations or disabled capabilities into a clean result.

Next, resolve the structured TypeScript interface, exact PHP call-site evidence and Rust resource/interface choice against actual rule needs, then exercise representative repositories and the first complete rule/evidence/judge/record/export path. D5's full initial language/rule scope remains a requirement; this qualification does not authorize silently reducing it.

Validation: 97 retained-result checks; real release-mode Oxc build and formatting check; successful analyzer executions with explicit expected-failure probes; fixture and artifact fingerprints; relocated evidence replay and recomputed statistics. No upstream analyzer full suite or production Anti-Slop suite was run.
