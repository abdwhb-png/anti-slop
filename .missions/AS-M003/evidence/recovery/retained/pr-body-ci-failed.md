## Summary

**Remote CI failed:** [Linux CI](https://github.com/abdwhb-png/anti-slop/actions/runs/43771124016/job/132987038769) rejects whitespace in the byte-preserved MDD raw logs and captured diffs. It stops before the build/product tests; the earlier local run did not include these then-untracked evidence files in Git's whitespace check. The draft is not ready to merge. No whitespace policy was weakened and no historical evidence was rewritten.

Add the AS-M003 library boundary for capturing selected Git changes and explicit supporting context as immutable, attributable inputs. Callers can choose staged, unstaged, local or an exact supplied base, retain the actual source/configuration bytes, and receive explicit failure when the capture is incomplete, unstable or exceeds its limits.

The implementation shares the existing configuration owner, preserves literal exclusions and exact rename attribution, and bounds file acquisition and owned Git subprocesses. This PR also includes the previously approved MDD v0.5 skill transition, mission preparation and retained execution evidence. No analysis verdict or new product CLI is introduced.

<!-- pr-demonstration:start -->
## Demonstration

A library caller can capture the intended Git comparison, inspect its exact before/after bytes and keep separate supporting context even after the files change.

### Follow the journey

1. **Library caller — choose the comparison and explicit context.** The real fixture gives `a.ts` four different contents: `base`, `new head`, `index` and `working`. The observed captures are:

   | Selection | Before | After | Separate context |
   | --- | --- | --- | --- |
   | Staged | HEAD: `new head` | Index: `index` | Working: `working` |
   | Unstaged | Index: `index` | Working: `working` | Working: `working` |
   | Local | HEAD: `new head` | Working: `working` | Working: `working` |
   | Explicit base | Supplied commit: `base` | Working: `working` | Working: `working` |

   The explicit base is resolved once and compared directly; the fixture distinguishes it from merge-base. [Actual observed output](https://github.com/abdwhb-png/anti-slop/blob/aab8746a39859edccd1e4326fef45c5e8bf0959e/.missions/AS-M003/evidence/P02/26-nonvisual-demo.log).

2. **Library caller — keep captured bytes after files change.** The public-boundary regression edits the originals after capture and verifies that retained source bytes, configuration, ranges and generation attribution still describe the captured input. [Executable integration scenarios](https://github.com/abdwhb-png/anti-slop/blob/aab8746a39859edccd1e4326fef45c5e8bf0959e/tests/workspace.rs), including `working_ranges_absence_and_configuration_survive_later_edits`.

3. **Library caller — reject excluded, missing or oversized inputs explicitly.** Excluded required context, missing objects and resource-limit violations produce explicit errors. The tests also verify that filters/helpers do not execute and owned stalled Git processes are cleaned up. [Capture regression results](https://github.com/abdwhb-png/anti-slop/blob/aab8746a39859edccd1e4326fef45c5e8bf0959e/.missions/AS-M003/evidence/P03/03-capture-regressions.log).

### Try it

No hosted trial applies: this change exposes a Rust library boundary. On Linux with the repository's pinned toolchain and dependencies available, check out this PR and run:

```sh
cargo test --frozen --test workspace four_comparisons_select_their_exact_sides_and_independent_context -- --exact --nocapture
cargo test --frozen --test workspace working_ranges_absence_and_configuration_survive_later_edits -- --exact
cargo test --frozen --test workspace exclusions_required_inputs_and_workspace_boundaries_are_explicit -- --exact
```

The first command prints the four comparisons above; the next two exercise retained inputs and explicit rejection through the real library.

### What remains limited

Capture is qualified locally on Linux. It does not claim a filesystem-wide atomic snapshot, analyzer execution, durable analysis runs, cold-disk performance or complete ADR-007 production-budget qualification. Controlled warm debug observations are retained with their sampling limits, including the slowest observed case. All admitted candidates count toward capture limits, including unchanged working inputs. No-Mistakes was not run. The current remote whitespace-policy failure remains unresolved.

<details>
<summary>Source, retained evidence and checks</summary>

- **Source:** PR head `aab8746a39859edccd1e4326fef45c5e8bf0959e`, based on `d75639c1f6e6a6fa75ca0b07e7895ba3b98f9add`. Validation originally ran on the uncommitted candidate over that base; the accepted source hashes were verified unchanged for this publication commit. Historical records retain their original revision and authority limits.
- **Checks:** the final local `scripts/ci/check.sh` passed: 70 Rust tests, 16 Python policy tests, formatting, build, Clippy, doctests, schema freshness and actual headless editor conformance. [Full retained CI log](https://github.com/abdwhb-png/anti-slop/blob/aab8746a39859edccd1e4326fef45c5e8bf0959e/.missions/AS-M003/evidence/P03/04-full-ci.log). Remote GitHub checks are separate and must be read on this PR's head.
- **Independent acceptance:** the fresh final reviewer accepted the complete corrected AS-M003 r2 outcome, independently exercised focused integration/mode/rename scenarios and checked source/evidence applicability. [Immutable final result](https://github.com/abdwhb-png/anti-slop/blob/aab8746a39859edccd1e4326fef45c5e8bf0959e/.missions/AS-M003/results/final-r2.json). Detailed worker reports referenced outside the worktree remain in the local task archive; this branch alone is not a portable archive of those external records.
- **TDD exception:** an earlier comparison extension was implemented before a valid RED. The user explicitly approved retaining it with the discrepancy recorded, complete checks and independent final review. [Exact approved exception](https://github.com/abdwhb-png/anti-slop/blob/aab8746a39859edccd1e4326fef45c5e8bf0959e/.missions/AS-M003/evidence/P02-tdd-exception-approved.json). Subsequent corrective changes have retained genuine RED/GREEN evidence.
- **Measurements:** [renewed controlled capture observations](https://github.com/abdwhb-png/anti-slop/tree/aab8746a39859edccd1e4326fef45c5e8bf0959e/.missions/AS-M003/evidence/P03); reproducible commands and limits are in the [README](https://github.com/abdwhb-png/anti-slop/blob/aab8746a39859edccd1e4326fef45c5e8bf0959e/README.md). These are observations, not production performance guarantees.

</details>
<!-- pr-demonstration:end -->

## Delivery status

Draft PR for review, with independent local acceptance retained. The mission remains `validating` until verified integration. Publication authority is separate from the earlier execution grant; merge is not authorized.
