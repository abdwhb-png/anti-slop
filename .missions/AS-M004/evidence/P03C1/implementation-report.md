# AS-M004 P03C1 report

done: correction candidate ready for examination

This bounded handoff corrects documentation and demo-test support only; production behavior and all prior evidence remain unchanged.

## Identity and Git provenance

Role: implementer for packet `AS-M004-P03C1` revision 1, mission `AS-M004` revision 2.
Execution identity: `as-m004-durable:s1794906814.562141.27268`, continued OpenCode session, route `opencode-go` / `deepseek-v4.1-flash`; high intent, no verified selectable effort variant.
Base and HEAD remain `45dac3c241338c94166369fe821fe1d3499cf974` (`fm/as-m004-durable`); all mission work remains uncommitted and no state, history, contract, packet, result or prior evidence was edited.
No commit, push, PR, merge, No-Mistakes run, dependency change, installation, reset, stash, clean or other-home write was performed.

## Changed paths

| Path | Change |
| --- | --- |
| `README.md` | Replaces the stale AS-M003 "integration pending" clause with verified merge through PR #3 into the starting baseline; puts the durable-records prose on one physical line per paragraph; states the database parent directory must already exist. |
| `docs/IMPLEMENTATION_ROADMAP.md` | Corrects the same stale AS-M003 current-status clause only; the historical plan and mission-state acceptance ownership are untouched. |
| `tests/records_demo.rs` | Reuses `support::hex` and `support::wait_bounded`, asserts successful child exit, removes the duplicated local hex helper and copied wait loop, and removes only the imports made unused. |
| `.missions/AS-M004/evidence/P03C1/` | This report, the current demo output and focused check evidence. |

No production source, Cargo manifest or lockfile changed.

## Corrections and rationale

The AS-M003 record correction is integrated through `https://github.com/abdwhb-png/anti-slop/pull/3`, merged into the baseline `45dac3c241338c94166369fe821fe1d3499cf974`; both touched current-status statements now say that instead of claiming integration was pending, while preserving the original PR #2 source-capture delivery and recovery provenance.
The added README durable-records prose now uses one physical line per paragraph; headings, the code block and list structure are preserved.
The example now states that the database parent directory must already exist; the store does not create it and no API changed.
The demo target now uses the existing test-support owners: `support::hex` for displayed bytes and `support::wait_bounded` for the fresh-process child, whose successful exit is asserted; the copied timeout loop and local hex implementation are gone.

## Validation

The real two-test demo was rerun after the change: `cargo test --frozen --test records_demo -- --nocapture` passed 2/2 and the fresh process still returned the exact selected bytes `776f726b696e672073656c65637465640a`, context `67756964650a`, working role and matching generations; the zero-budget refusal left the prior record readable and origins remained removed. Raw output is `demo-output.txt`.
Focused diagnostics passed: `cargo clippy --frozen --test records_demo -- -D warnings`, `cargo fmt --all -- --check`, and `git diff --check`; outputs are `clippy-demo.txt`, `format-check.txt` and `diff-check.txt`.
No RED was manufactured: this is documentation and test-support reuse with no production behavior change; the prior demo already passed and remains the executable before/after.

## Prior qualification applicability

P03 production, failure, path and resource qualification remains applicable because no production source, dependency, manifest, lockfile or measurement input changed in P03C1; only README, the roadmap status clause and demo test support changed.
The prior records (38) and full CI receipt remain the last production qualification for the unchanged source at this base; this packet intentionally did not rerun the whole CI suite, and GitHub CI will cover the final committed PR head as required by the handoff.
The only renewed runtime evidence is the real demo above; P03 measurements and failure logs were not rewritten.

## Limits and return

The demo fixture commit identity and generation vary per run because it creates a fresh temporary Git repository; the reported bytes, roles and equality checks are stable and were observed in this run.
No new product guarantee, API, dependency, state edit or independent review is included; Firstmate examines this bounded correction and Codex remains the fresh final whole-mission reviewer.
