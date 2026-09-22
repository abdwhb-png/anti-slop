---
name: anti-slop-analyzer-qualification
description: Qualify an Anti-Slop language adapter with reproducible native facts, source-generation coherence, explicit capability gaps and complete latency/memory accounting. Use when investigating or changing an analyzer integration; skip ordinary builds, metadata-only dependency edits and documentation version corrections.
---

# Anti-Slop Analyzer Qualification

## When to Use

Use this workflow when evaluating a language analyzer, changing an integration boundary or resolving uncertainty about its facts, source consistency or operational cost. Scope it to a concrete capability or invalidated qualification claim.

## When Not to Use

Skip it for a routine Rust build, README correction or manifest metadata update that does not claim new analyzer behavior. A documentation-only request does not authorize installation or native execution. A working CLI or successful LSP response alone is not a reason to declare an adapter qualified.

## Define the claim and preserve the experiment

A broad “supports this language” claim hides missing capabilities. Select the smallest decisive experiment so evidence answers the actual integration question.

1. Read [ADR-002](../../../docs/decisions/002-replaceable-native-language-analyzers.md), the relevant existing research, and [ADR-007](../../../docs/decisions/007-performance-budgets-and-qualification.md) when resources are involved.
2. State the required fact or lifecycle behavior, language/version, fixture/workload scope, expected source mapping and failure conditions. Separate extraction, coherence, resource and rule-quality claims.
3. Preserve previous probes, manifests and archives. Put a new authorized experiment in an isolated temporary setup or a new evidence directory; do not update an old lockfile and keep its old result label.
4. Identify exact binary/package/toolchain/configuration inputs, source provenance and checksums where available. Inspect established dependency controls before installation. Existing temporary permissions are bounded to their original scope; do not bypass an unavailable required control.
5. Use fixtures or controlled captures. Do not alter the user's active repository, install tooling into analyzed projects or trigger a launcher's implicit downloader. Installation and paid inference are separate from a read-only evaluation request.

## Exercise actual native capabilities

The useful output is a fact with scope and source identity, not transport success. Check each capability that a rule will rely on through the intended native boundary.

1. Query representative cross-file declarations, references, types or delegation using the real integration. Add the relevant legitimate boundary and partial/unresolved case.
2. Compare expected facts and exact source spans against captured bytes, including position encoding and line endings. Textual macro expansions or approximate matches are not exact original-source mappings.
3. Record unsupported language features, disabled build scripts/macros, framework behavior and dynamic dispatch. Keep partial facts useful without interpreting an empty unresolved response as absence.
4. Test the failure that could invalidate the claim: crash, timeout, malformed response or missing capability. Retry only within the task's stated bounds and preserve failed attempts.

## Prove generation coherence

An analyzer can read a different state from the Rust evidence collector. The intended result is a declared consistency boundary with explicit gaps, because plausible facts from the wrong generation cannot support a current finding.

1. Capture selected dirty/untracked source bytes plus relevant configuration and dependency identities. Record native overlay text/version when used.
2. Establish what the analyzer actually consumed. Idle notifications and equal pre/post disk hashes are insufficient when overlays or external dependencies can change semantics.
3. Exercise a generation transition and a late response; old results may remain historical but must not overwrite current output. Include a changed/new/deleted caller when absence or reference caches matter.
4. Exercise configuration/dependency changes for the scope claimed. Inputs outside the managed view must be bound or declared unresolved; a source-only fixture cannot establish hermetic dependency coverage.
5. Distinguish a coherent captured state from freshness against the client's current state. Record unknown/stale freshness rather than extrapolating it from one readiness signal.

## Measure the claimed resource boundary

Query-only timing and core-only RSS can disguise the cost of a native helper. Compare only equivalent work, with the full boundary needed by the claimed budget.

1. Separate startup, warm local analysis, available-report reads, repeated edits/cancellation and retained idle state. Declare fixture sizes, rules, evidence coverage and cache conditions.
2. For local-path measurements, include capture, invalidation, evidence, recording and serialization; controlled provider replay excludes remote waiting only. Also report real end-to-end time when live inference is actually run.
3. Measure the entire owned process tree, including workers and database activity. Include retained idle memory as its own observation after work completes; startup/active peaks cannot establish it. State sampling method, missed-peak/shared-page limits, run count, p50/p95 and raw observations. A core process snapshot cannot establish a memory target.
4. If resident lifecycle is claimed, verify normal client close and abrupt parent death on the supported Linux/WSL2 environments. Bound queues and cleanup; successful graceful shutdown is not sufficient by itself.
5. Compare each available observation with the corresponding numeric target from ADR-007 and name missing boundaries separately. For example, a TypeScript core retaining 80 MiB and its helper 300 MiB is already a sampled total above the 256 MiB target; sampling caveats limit exact accounting but do not justify describing every budget as untested. Distinguish observed over-budget evidence from unmeasured peaks or latency paths.
6. Use only the relevant resource scenarios during an experiment. Full representative qualification belongs after integration stabilizes; do not rerun the complete corpus after every routine edit.

## Report acceptance without extrapolation

Return a claim-by-claim matrix with observed evidence, limitations and one of: established within scope, failed, partial, or not tested. Include exact inputs, commands, failures, raw artifact locations when retained, and remaining work. A fixture result does not establish production quality, a performance pass does not establish rule accuracy, and configured limits do not prove enforced resource guarantees.
