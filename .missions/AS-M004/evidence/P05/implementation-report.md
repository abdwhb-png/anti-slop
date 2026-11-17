# AS-M004 P05 — raw evidence whitespace classification

Complete implementation report for packet AS-M004-P05 revision 1 under mission AS-M004 revision 2. Outcome: the cumulative Git whitespace gate is green again because the retained raw evidence of this mission is classified through the repository's existing Git attributes owner, without touching a single historical or product byte, and without weakening any authored-source check. No production defect is implied or repaired.

## Assignment and exact state

Actor `as-m004-durable:s1794918722.2688789.30962`, role implementer. Repository `anti-slop` at `/home/abdwhb/engineering/firstmate-orchestration-test/data/as-m004-durable/worktree`, branch `fm/as-m004-durable`, HEAD `5e19368c98fb46410b9d258bffa912fa9bac83e2`, sole parent and mission base `45dac3c241338c94166369fe821fe1d3499cf974`. Packet `AS-M004-P05` revision 1, mission revision 2, delivery assessment `continue`.

Authority for this bounded correction is the packet plus `.missions/AS-M004/evidence/execution-authority.txt`: the L04 grant covers implementation, checkpoint/candidate commits and one GitHub PR targeting `main`; merge, No-Mistakes, provider or account changes and daily-home modification remain unauthorized, and this packet narrows that grant further. Nothing was committed, pushed, opened, merged, installed, deployed or cleaned. The existing transport commit was preserved; no amend, reset, stash, clean or discard was used.

## Root cause, reproduced

The failing gate is `git diff --check`, reached from `scripts/ci/check.sh` line 14 through `scripts/ci/policy.py` lines 271–280, which runs `git diff --check <base>`. Git's whitespace check only inspects content that Git can see: tracked working-tree files and index entries. It never inspects untracked files.

The transport changed 177 files: 172 added and 5 modified (`Cargo.lock`, `Cargo.toml`, `README.md`, `docs/IMPLEMENTATION_ROADMAP.md`, `src/lib.rs`). The 172 added files were untracked until staging, so the earlier clean `git diff --check` never inspected them; the 5 tracked modifications were inspected then and were clean. The transport's `git diff --cached --check` failed with 101 diagnostics only after staging made the new archive visible. The transport report records that sequence; this packet reproduces the mechanism rather than trusting the description.

With the mission archive indexed, the gate reports exactly 101 diagnostics in 47 files, exit 2:

| Class | Count | Files |
| --- | ---: | --- |
| `blank-at-eol` only | 59 diagnostics | 10 |
| `blank-at-eof` only | 36 diagnostics | 36 |
| both classes | 6 diagnostics | 1 |

The 47 files are retained unified diffs (`Cargo.lock.diff` and `tracked-source.diff` for each packet, whose blank context lines carry a leading space), captured test and measurement transcripts that end with a terminal blank line, and the frozen conception and preparation copies. The product delta is not implicated: no path under `src/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `README.md`, `docs/IMPLEMENTATION_ROADMAP.md` or the mission contract appears in any diagnostic.

The same masking condition recurred live inside this packet, which is the strongest available confirmation. The copied `transport-commit-report.raw.txt` quotes the original failing gate, so it carries 64 `blank-at-eol` lines. While it was untracked the worktree check stayed green. The moment it was staged with the rest of the P05 evidence, the staged gate returned exit 2 with exactly 64 diagnostics in that one file. Classifying it cleared the gate. That is the same cause, observed again, not a second defect.

## Counterfactual: the attribute source is the only variable

Git resolves the `whitespace` attribute per path and, by default, from the working tree. Forcing the source to the committed tree isolates the attribute change from every content change:

```
git --attr-source=HEAD diff --cached --check 45dac3c241338c94166369fe821fe1d3499cf974
exit 2, 101 diagnostics, byte-identical to the pre-edit capture
git diff --cached --check 45dac3c241338c94166369fe821fe1d3499cf974
exit 0
```

The same index, the same base and the same content produce a different result only because the attribute source differs. `attributes-semantics.txt` shows the same contrast per path: `git --attr-source=HEAD check-attr whitespace` reports `unspecified` for the flagged raw files, while the working-tree attributes report the narrow class exclusion.

## The correction

`.gitattributes` keeps its existing `*.patch -whitespace` line and gains 50 path-scoped entries: the 47 historical files above and the three P05 raw transcripts. Each entry disables only the class actually observed in that exact file. Thirty-six files carry `whitespace=-blank-at-eof`, ten carry `whitespace=-blank-at-eol`, one (`P04/measurement-source.diff.txt`) carries both, and the three P05 transcripts carry `-blank-at-eol`.

The semantics stay narrow on purpose:

- `space-before-tab` is never disabled anywhere.
- Every path not listed keeps Git's complete default set, including all authored source, tests, manifests, `README.md`, the roadmap, the mission contract and every authored Markdown report.
- No pattern, glob or directory-wide rule was introduced, so a future file cannot inherit an exclusion by accident.
- `conception-accepted.md` is the only `.md` entry, and only its `blank-at-eof` class is disabled; a trailing space in that file still fails (case 5 below).
- `.missions/AS-M004/evidence/P01/preparation.diff` is deliberately absent, so the existing unit test that requires it to reject whitespace keeps passing.

The declared set was compared with the observed set mechanically: the 47 historical entries equal the 47 flagged paths exactly, with no missing path, no extra path and no entry disabling a class that file did not exhibit. The only other rule in the file is the pre-existing `*.patch -whitespace`.

No historical file was renamed, rewritten, trimmed, re-wrapped, deleted or compressed. No Git configuration, `.git/info/attributes`, `scripts/ci/policy.py`, workflow or CI policy changed, and no new validator or abstraction was introduced. The classification is configuration only.

One deliberate, reviewable choice: the entries live in the root `.gitattributes`, because the packet's writable scope names `.gitattributes` and new `evidence/P05/` material. AS-M003 used a mission-local `.missions/AS-M003/evidence/.gitattributes`; an equivalent AS-M004 file would have been outside this packet's scope. The entries are path-explicit, so the location affects readability, not behavior.

## Before and after, with real exits

| Command | Before | After |
| --- | --- | --- |
| `git diff --check 45dac3c…` | exit 2, 101 diagnostics in 47 files | exit 0 |
| `git --attr-source=HEAD diff --cached --check 45dac3c…` | exit 2, 101 diagnostics | not applicable (pre-correction index) |
| `git diff --cached --check 45dac3c…` | exit 2, 101 diagnostics, then 64 more when the P05 raw report was indexed | exit 0 |
| `python3 scripts/ci/policy.py whitespace --base 45dac3c…` | not run (underlying `git diff --check` exited 2) | exit 0 |

Captures: `before-whitespace-check.txt` (168 lines, the full failed output and exit), `before-staged-check.txt` (the same diagnostics through the committed attribute source), `after-staged-check.txt` and `policy-whitespace-check.txt` (exit 0, empty output). The policy command was not executed before the edit; its pre-correction result is the same `git diff --check` comparison already captured byte-for-byte, and the wrapper was read rather than assumed.

The staged cumulative check is the decisive one, because only it includes newly indexed files. The worktree check cannot see untracked files, which is precisely the masking condition this packet corrects.

## Negative controls

`negative-controls.sh` rebuilds a controlled temporary Git repository that contains only the real edited `.gitattributes` and the real `.missions/AS-M003/evidence/.gitattributes` as configuration, with `core.attributesFile=/dev/null` and Git's default whitespace set. It then runs the real `scripts/ci/policy.py whitespace` command against that repository; no policy logic is copied. `negative-controls.txt` is the full transcript.

| Case | Fixture | Expected | Observed |
| --- | --- | --- | --- |
| 1 | real `P03/tracked-source.diff`, 12,701 bytes | permitted | exit 0 |
| 2 | new `src/new_source.rs` with a trailing space | rejected | exit 1, `trailing whitespace.` |
| 3 | authored `.missions/AS-M004/evidence/P05/authored-report.md` with a trailing space and blank EOF | rejected | exit 1, both classes |
| 4 | unlisted `.missions/AS-M004/evidence/P01/preparation.diff` with a blank context line | rejected | exit 1, both classes |
| 5 | exempted `conception-accepted.md` with a trailing space but no blank EOF | rejected | exit 1, `trailing whitespace.` |
| 6 | `P04/measurement-source.diff.txt` carrying both observed classes | permitted | exit 0 |

Case 1 also proves byte preservation: the working-tree, committed and repository copies all hash to `67c2218a521d7d495f78fd880c407682f3356be926282d7f5149ccb609c7c493`. Case 3 is the control that matters most for this mission: an authored Markdown report inside the very same `P05` directory still fails, so the classification did not create a permissive zone around the evidence.

## Unchanged bytes

Preservation was verified by hash, not by assertion. `preservation-hashes.txt` holds the table produced before any edit and again after the attribute edit; the two runs are byte-identical, table SHA-256 `f95dafded7b539e5c3d60bb9858805bd8f695abeb378cc5d277b5831afea4912`.

- All twelve `final-r2` `sourceScope.reviewedPaths` match their recorded SHA-256 and byte counts exactly, including `src/records/mod.rs`, `src/lib.rs`, the three test targets, the support module, the C harness, both manifests, `README.md`, the roadmap and the mission contract.
- The demonstration inputs still match their recorded digests: `P03C1/demo-output.txt` `28b5f562…a6aa0`, `P04/implementation-report.md` `22b68713…f773`.
- The 47 historical raw files are unchanged byte for byte.
- The copied `transport-commit-report.raw.txt` is unchanged, SHA-256 `ed6908949ba95826f80e23766b0621793db38c5424bd58be787fa5de4c85d38f`, 24,061 bytes.
- Comparing every tracked file at HEAD with its working-tree content reports exactly three differences: `.gitattributes`, plus `state.json` and `history.jsonl`, which were already modified before this packet began (packet registration) and were not touched here.

`.gitattributes` grew from 84 bytes to 4,824 bytes, SHA-256 `4278f0afcf40a066b9f5686f972004f83ba49fb850efc07b0951ebe072db337c`.

## Existing checks

- `python3 -m unittest discover -s scripts/ci -p 'test_*.py' -v`: 19 tests, `OK`, exit 0 (`policy-unit-tests.txt`). This is the repository's existing CI policy target, and it includes `test_maintained_files_and_unrelated_evidence_still_reject_whitespace`, which asserts that `.missions/AS-M004/evidence/P01/preparation.diff`, maintained files and unrelated evidence still reject whitespace, and `test_raw_evidence_keeps_checks_for_other_whitespace_classes`, which asserts that an exempted raw file still rejects the other classes.
- `python3 scripts/ci/policy.py whitespace --base 45dac3c…`: exit 0 (`policy-whitespace-check.txt`).
- `node …/mdd-bundle/scripts/validate.mjs .missions/AS-M004/state.json --json`: `{"valid":true,"issues":[]}`, exit 0 (`mdd-state-validation.txt`).

## Demonstration applicability

The nonvisual journey is unchanged and remains the recorded library journey: select `a.ts` and retain `guide.ts` context, save synchronously, refuse a second save under a zero budget while the first capture stays readable, remove the origins, then read exact working and context bytes, roles and generations in a fresh process. All three scenes are `observed` against `.missions/AS-M004/evidence/P03C1/demo-output.txt`; every evidence, report and applicability digest in the current demonstration record was re-verified against the files and matches, including the P05 applicability report `68f5413ffc3d9489123e26141716ad2ccbe7b4c16de951415aeb60232a75eb2f`.

No scene was re-executed and no media was produced. This packet changes no demonstration input, fixture, dependency or retained output, so the existing scenes remain applicable under the explicit reuse reason already recorded. This is not a new runtime observation and not an acceptance of the demonstration.

## Limits, uncertainty and checks not run

- CI itself was not run. The classification takes effect at the published head only, so the GitHub run must still happen there; a local green gate is not a CI result.
- No Rust, Node, resource, fault or concurrency suite was rerun, and no build, clippy or formatting check was run, because no source, manifest, dependency or test byte changed and the hash tables prove it. Only the whitespace and policy-configuration surfaces were exercised.
- The 50 entries are archive-specific by design. A raw file added later with the same bytes will fail the gate until it is classified explicitly. That is the intended behavior, not a remaining gap in this correction.
- Three P05 raw transcripts are exempted for `blank-at-eol` because they reproduce the failing output verbatim, including the content lines Git echoed. This is disclosed rather than hidden: they are raw transcripts, every other P05 file (this report, the logs, the script) is fully checked, and case 3 proves an authored report in the same directory still fails.
- `.missions/AS-M004/packets/P05.md` and the state and history edits are outside this packet's staging authority, so they are not in the index. The packet file is untracked and therefore invisible to `git diff --check`; a supplementary `git diff --no-index --check -- /dev/null .missions/AS-M004/packets/P05.md` produced no diagnostic (exit 1 only because the two inputs differ).
- Whether the root location of the entries should later move to a mission-local file, matching the AS-M003 layout, is a presentation choice for the reviewer; behavior does not depend on it.

## Index and worktree state

The index contains exactly the correction and its evidence: `.gitattributes` modified, and thirteen new files under `.missions/AS-M004/evidence/P05/`. `index-state.txt` records the exact staged name-status list, including the 177 previously committed mission files that the cumulative comparison also re-checks. Nothing is committed. The worktree additionally holds Firstmate's pre-existing `state.json` and `history.jsonl` modifications and the untracked packet file. `git diff --cached --check` and `git diff --check` against the mission base both exit 0 with this index.

done: candidate ready for examination

This return is a candidate for examination. It does not claim packet acceptance, mission validation, publication readiness, merge or delivery.
