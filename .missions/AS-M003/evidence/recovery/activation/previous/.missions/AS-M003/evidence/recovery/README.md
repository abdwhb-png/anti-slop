# AS-M003 historical-reference recovery

This is the dated record of the authorized AS-M003 reference correction. The capture implementation was independently accepted and merged through [its delivery](https://github.com/abdwhb-png/anti-slop/pull/2). This local record/document patch is under firstmate review, uncommitted and unpublished. The [active state](../../state.json) remains validating at revision 20; the proposed completion transition has not been activated. AS-M004/L04 remains set aside.

## Authority and unchanged historical meaning

The [2026-11-17 authority](authority/repair-authority.md) records the exact answer: “Corriger les 48 chemins et archiver les originaux (recommandé)”. It explicitly authorizes preserving original bytes and correcting the identified historical paths. This is an exception to historical reference-byte immutability, with original records archived below; it is not a skill migration, new product decision, independent product review or backdated merge grant. The [bounded activity observation](authority/pre-repair-activity.json) records the named old endpoint and four recorded reviewer PIDs as absent before repair. No old-home write or lifecycle control occurred.

The old publication grants and actual head/CI/merge chain are retained in [integration proof](integration-proof.json), backed by the [supplied primary forge observations](forge/forge-observations.json) and [identical-tree proof](forge/merge-source-proof.json). Historical merge permission remains unlocated. The current authority acknowledges the existing merge at the present date and allows evidence-based record reconciliation; it does not invent an earlier decision. The original [independent final-r3 verdict](../../results/final-r3.json), reviewer identities, negative results, bounded P02 TDD exception and original failed CI remain historical evidence.

## Exact archive and path repair

The original layout placed the repository in a task's `worktree/` directory and retained coordinator/reviewer evidence beside it. References using three or four parent components therefore escaped a normal clone. The installed frozen validator reproduced 39 missing-reference diagnostics on state; direct validation of final-r2 found nine additional references. These are 48 occurrences in 11 artifacts pointing to 40 distinct files.

[Originals manifest](originals-manifest.json) identifies the exact pre-repair state, all 11 affected artifacts, the complete 20-event journal, and the two current-status documentation originals: 14 files with SHA-256 and byte counts. [Path map](path-map.json) records each before/after reference and source line, original artifact hash and path-only repaired hash. Historical artifacts changed only those 48 path strings; modified files are not claimed to retain their original bytes. Original journal bytes, actor names, timestamps, outcomes, approval wording and failed attempts remain recoverable from `originals/`.

[Retained manifest](retained-manifest.json) identifies 118 exact copied files: the 40 required targets and 78 supporting reports, logs, command receipts, source snapshots, manifests and two archives. Original source/archive/report bytes are unchanged. The [parent-evidence manifest](parent-evidence-manifest.json) binds the current authority/activity and supplied forge proofs. [Source applicability](source-applicability.json) binds the eight accepted source/fixture/README hashes observed before status-document edits to the merged tree. Product source, tests, approved contracts, AGENTS, the frozen skill and dependencies are unchanged.

## Historical links and limits

[Historical link inventory](historical-links.json) preserves relocation mappings and classifications for structured file-reference fields and Markdown links without rewriting raw records. Old absolute paths and narrative links remain original data. Repository counterparts provide navigation, not automatic byte equality for older generations; use the recorded hashes, archived originals and rejected/accepted source snapshots to distinguish them. The frozen workflow link maps to the identical installed bundle rather than a copied second skill.

One narrative dependency in the negative final-r1 report points to `../AS-M003-preflight/report-after-recorder.md`, outside the authorized retained task. It was not read or copied; the published prerequisite verdict remains available in the [existing prerequisite record](../prerequisites-accepted.json). This package does not claim to reconstruct the separate preflight task or every historical environment. Compiled executables, caches and ephemeral fixtures were not bulk-copied. Command arguments and arbitrary paths inside stdout/source are historical data, not a promise to recreate the old filesystem. The actual P04 local CI log and its RED/GREEN command-record/log dependencies are retained and hash-checked.

Before copying, a bounded signature scan covered retained files, parent evidence, archived originals and regular members of both source archives. The [scan receipt](checks/secret-scan.json) records zero findings and its limitations; it is not a guarantee against every secret encoding. Raw evidence remains byte-exact. Any per-file whitespace attributes in this directory name only observed raw-file error classes and preserve the original bytes.

## Reproduce the record checks

From the repository root, without dependencies or application tests:

```sh
python3 .missions/AS-M003/evidence/recovery/checks/verify-recovery.py
node .agents/skills/mission-driven-dev/scripts/validate.mjs .missions/AS-M003/state.json --json
node .agents/skills/mission-driven-dev/scripts/validate.mjs .missions/AS-M003/results/final-r2.json --json
node .agents/skills/mission-driven-dev/scripts/validate.mjs .missions/AS-M003/state.json --candidate .missions/AS-M003/evidence/recovery/closure-candidate.json --json
```

The check receipts retain complete argv, working directories, timestamps, exits and stdout/stderr. Before-state/final-r2 failures and after-repair successes are separate records. Portability is checked in a plain exported directory containing tracked files and intended additions, with no `.git`, symlinks or task-parent evidence. The candidate is checked against the repaired actual state and its exact journal prefix; validating a candidate does not publish it or authorize progression.

Firstmate must examine the complete patch, preservation checks, portability receipt and [closure candidate](closure-candidate.json) before activation. The proposed state retains the original independent final-r3 acceptance, adds recovery/integration proof and appends one dated completion event without recreating checkpoint commits. Commit, publication and further mission work require their separate authority. No Rust/application tests, benchmark, model call, package operation, remote request, commit, push or merge ran for this record correction.
