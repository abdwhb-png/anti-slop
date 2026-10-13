# Task: <one observable outcome>

> Standalone task template. For mission-based work, use the current Mission Contract and supplied workflow linked by the [roadmap](../IMPLEMENTATION_ROADMAP.md). This template is not a Mission Contract or a Task Packet; derive packets only just before authorized dispatch. Existing task documents remain historical records.

Use this template for a bounded unit of work another agent can finish and verify. Remove irrelevant fields; do not expand a small task into an entire release. Write the task in English.

## Intent and authority

- Requested outcome and why it matters:
- Task type: documentation / qualification / implementation / review.
- Relevant accepted ADRs and any explicitly approved exception:
- Missing decisions, if any, and independent work that can proceed:

## Starting state

- Current checkout/revision and relevant existing changes:
- Implemented public boundary and exact files/modules that own it:
- Relevant research, version pins and limits of that evidence:

## Scope

- One deliverable or vertical slice:
- Files/modules expected to change:
- Explicit exclusions:
- Dependency or external actions needed, with existing authorization where applicable:

## Acceptance and validation

Describe observable behavior, including a legitimate counterexample and incomplete/failure handling when relevant. Do not prescribe assertions that merely copy the implementation.

- Positive case:
- Counterexample or boundary case:
- Failure/uncertainty case:
- Smallest executable check and command:
- For behavior changes: intended RED, then GREEN and focused regression checks.
- For documentation/configuration: applicable consistency or before-and-after checks.
- Broader checks needed because of this task's actual scope:

## Completion and handoff

- Changed behavior/artifacts and evidence of acceptance:
- Checks actually run, results, and checks not run:
- Incomplete work, unresolved questions and a bounded next task:
- Commit and activation state, reported separately:

Do not mark an ADR implemented merely because a fixture passes. Do not reduce the first-release rule/language scope when completing a smaller slice.
