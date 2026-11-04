# Task: Repository-wide CI with reuse after validated PR merges

## Intent and authority

The user approved the implementation plan on 2026-11-04: one required Linux
check for the whole repository, automatic coverage of new workspace tests,
qualified downloads, exact-tree reuse after PR merges, and full verification of
direct pushes. The user selected administrator bypass to allow owner direct
pushes and accepted that this can also bypass PR checks. Authority includes a
separate commit on PR #1 and enabling the required check after it passes. It
does not authorize merging PR #1 or starting another product mission.

## Starting state and boundary

Implementation starts at `3e3119a1acd71cec581cef8c276e1eb935170794` on
`recover/as-m002`, with a clean worktree and no existing workflow. Rust is pinned
to 1.96.1; Node is required by the current conformance tests. README owns the
dependency controls: official metadata, a seven-day release-age window, locked
integrity and Socket Firewall for new downloads.

The workflow adapts GitHub events; `scripts/ci/` owns the shared verification,
dependency policy and receipt decisions. These do not implement analyzer rules,
policy verdicts, release publishing or mission acceptance. Existing research
manifests and mission history are preserved. Unified-patch context spaces are
preserved through Git attributes while source whitespace stays checked.

## Acceptance and validation

Successful PR verification records the actual checked-out tree and run identity.
Only a successful current PR attempt matching the merged PR's final head and
exact main tree can replace full verification. No proof, direct pushes,
cancelled/failed attempts, malformed data, expired artifacts and API errors all
select full verification. Receipts are bounded JSON data, never extracted or
executed. Authentication is stripped on cross-host artifact redirects.

Focused tests exercise actual Git merge/squash/rebase trees and the production
decision boundary with external API fixtures. Dependency tests cover eligible,
too-young, yanked, missing and integrity-mismatched releases, unavailable metadata
and absent comparison history. `bash scripts/ci/check.sh` passes locally; the
first Actions execution and receipt must also be inspected before claiming
runner compatibility. Concurrency configuration needs live confirmation when
two runs overlap; reuse after a real main merge remains pending the first
separately authorized merge.

## Completion and handoff

Use PR #1's current checks and job logs as the canonical live evidence; avoid
new documentation commits solely to record passing CI and trigger another run.
Keep implementation, published workflow and activated branch protection distinct.
At implementation time, this workspace's HTTP policy denies `api.github.com`
to `gh`, and the GitHub connector has no branch-protection mutation. Resolve that
access before applying required-check settings; do not claim enforcement from
workflow YAML alone. All code and local verification can proceed independently.
