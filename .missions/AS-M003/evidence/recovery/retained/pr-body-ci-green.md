## Summary

Add the AS-M003 library boundary for capturing selected Git changes and explicit supporting context as immutable, attributable inputs. Callers can choose staged, unstaged, local or an exact supplied base, retain the actual source/configuration bytes, and receive explicit failure when the capture is incomplete, unstable or exceeds its limits.

The implementation shares the existing configuration owner, preserves literal exclusions and exact rename attribution, and bounds file acquisition and owned Git subprocesses. This PR also includes the previously approved MDD v0.5 skill transition, mission preparation and retained execution evidence. No analysis verdict or new product CLI is introduced.

The CI publication correction preserves every historical evidence byte. Native Git attributes except only the observed whitespace class for each of 34 named raw evidence files; source, documentation, maintained metadata, unrelated evidence and other whitespace classes retain their checks. Three integration regressions exercise the actual policy entrypoint in real Git repositories. [Correction and byte-preservation audit](https://github.com/abdwhb-png/anti-slop/blob/1560c240d30711865bb01e039bad03c10f865662/.missions/AS-M003/evidence/P04/13-return-audit.json).

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

   The explicit base is resolved once and compared directly; the fixture distinguishes it from merge-base. [Actual observed output](https://github.com/abdwhb-png/anti-slop/blob/1560c240d30711865bb01e039bad03c10f865662/.missions/AS-M003/evidence/P02/26-nonvisual-demo.log).

2. **Library caller — keep captured bytes after files change.** The public-boundary regression edits the originals after capture and verifies that retained source bytes, configuration, ranges and generation attribution still describe the captured input. [Executable integration scenarios](https://github.com/abdwhb-png/anti-slop/blob/1560c240d30711865bb01e039bad03c10f865662/tests/workspace.rs), including `working_ranges_absence_and_configuration_survive_later_edits`.

3. **Library caller — reject excluded, missing or oversized inputs explicitly.** Excluded required context, missing objects and resource-limit violations produce explicit errors. The tests also verify that filters/helpers do not execute and owned stalled Git processes are cleaned up. [Capture regression results](https://github.com/abdwhb-png/anti-slop/blob/1560c240d30711865bb01e039bad03c10f865662/.missions/AS-M003/evidence/P03/03-capture-regressions.log).

### Try it

No hosted trial applies: this change exposes a Rust library boundary. On Linux with the repository's pinned toolchain and dependencies available, check out this PR and run:

```sh
cargo test --frozen --test workspace four_comparisons_select_their_exact_sides_and_independent_context -- --exact --nocapture
cargo test --frozen --test workspace working_ranges_absence_and_configuration_survive_later_edits -- --exact
cargo test --frozen --test workspace exclusions_required_inputs_and_workspace_boundaries_are_explicit -- --exact
```

The first command prints the four comparisons above; the next two exercise retained inputs and explicit rejection through the real library.

### What remains limited

Capture is qualified locally on Linux. It does not claim a filesystem-wide atomic snapshot, analyzer execution, durable analysis runs, cold-disk performance or complete ADR-007 production-budget qualification. Controlled warm debug observations are retained with their sampling limits, including the slowest observed case. All admitted candidates count toward capture limits, including unchanged working inputs. No-Mistakes was not run.

<details>
<summary>Source, retained evidence and checks</summary>

- **Source:** PR head `1560c240d30711865bb01e039bad03c10f865662`, based on `d75639c1f6e6a6fa75ca0b07e7895ba3b98f9add`. The capture source and fixtures remain byte-identical to the independently accepted implementation. The CI correction and its regression coverage were checked on the current candidate; committed source applicability and the remote run were verified separately. Historical records retain their original revision and authority limits.
- **Checks:** the final local `scripts/ci/check.sh` passed: 70 Rust tests, 19 Python policy tests, formatting, build, Clippy, doctests, schema freshness and actual headless editor conformance. [Current local CI receipt](https://github.com/abdwhb-png/anti-slop/blob/1560c240d30711865bb01e039bad03c10f865662/.missions/AS-M003/evidence/P04/10-stable-ci.json). [Remote Linux CI](https://github.com/abdwhb-png/anti-slop/actions/runs/43775569101) passed for this PR head, including the preserved raw evidence. The original failed run remains historical evidence.
- **Independent acceptance:** the fresh final reviewer accepted the complete corrected AS-M003 r2 outcome after the CI correction, exercised the affected policy boundary and checked applicability of unchanged capture and measurement evidence. [Immutable final result](https://github.com/abdwhb-png/anti-slop/blob/1560c240d30711865bb01e039bad03c10f865662/.missions/AS-M003/results/final-r3.json). Detailed worker reports referenced outside the worktree remain in the local task archive; this branch alone is not a portable archive of those external records.
- **TDD exception:** an earlier comparison extension was implemented before a valid RED. The user explicitly approved retaining it with the discrepancy recorded, complete checks and independent final review. [Exact approved exception](https://github.com/abdwhb-png/anti-slop/blob/1560c240d30711865bb01e039bad03c10f865662/.missions/AS-M003/evidence/P02-tdd-exception-approved.json). Subsequent corrective changes have retained genuine RED/GREEN evidence.
- **Measurements:** [renewed controlled capture observations](https://github.com/abdwhb-png/anti-slop/tree/1560c240d30711865bb01e039bad03c10f865662/.missions/AS-M003/evidence/P03); reproducible commands and limits are in the [README](https://github.com/abdwhb-png/anti-slop/blob/1560c240d30711865bb01e039bad03c10f865662/README.md). These are observations, not production performance guarantees.

</details>
<!-- pr-demonstration:end -->

## Delivery status

Draft PR for review, with independent local acceptance retained. The mission remains `validating` until verified integration. Publication authority is separate from the earlier execution grant; merge is not authorized.
