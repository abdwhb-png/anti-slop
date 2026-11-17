# Thread-child observation regression — evidence

Actor: `as-m004-durable:s1794916284.2288889.8486`. Repository `anti-slop`, mission AS-M004 revision 2, packet AS-M004-P04 revision 1. Branch `fm/as-m004-durable`, base and HEAD `45dac3c241338c94166369fe821fe1d3499cf974`. All L04 source remains uncommitted exactly as the final-r1 review recorded it; the only modified deliverable in this packet is `tests/records_measure.rs`, which is untracked and therefore still diffed against the preserved pre-packet copy rather than against Git.
Toolchain: `rustc 1.96.1 (31fca3adb 2026-06-26)`, `cargo 1.96.1 (356927216 2026-06-26)`, Node `v26.13.0`. Host: `Linux 7.0.0-42-generic x86_64 GNU/Linux`, 4 available CPUs.

## Preserved pre-packet source

`measurement-source-before.raw.rs.txt` is byte-identical to the source the final-r1 review examined: 15,382 bytes, SHA-256 `73ffa7ac07559ea87f2d91f70850fa6a9957e3235a5c5b80aa7976858dfa5379`, matching the packet and the final-r1 `reviewedPaths` entry. Its `tree_rss` reads only `/proc/<pid>/task/<pid>/children` for every process it visits.

## Fixture repair before counting RED

The first regression attempt failed for a fixture reason rather than the intended omission, so it is not counted as RED. The launcher thread exited immediately after spawning Git, and the kernel reparented the child to another thread of the test process; the child was then correctly absent from the (now exited) launcher's children list. The fixture now holds the launcher thread alive on a release channel for the whole observation, and the test asserts the live child is listed under that nonleader TID before measuring. The kernel's within-thread-group reparenting is also why a leader-only reader can appear empty even when the process clearly owns children.

## RED — the omitted child behavior

Command: `/home/abdwhb/.cargo/bin/cargo test --frozen --test records_measure -- --exact tree_rss_includes_git_children_launched_from_a_nonleader_thread --nocapture`. Exit 101. Raw output: `red-01-thread-child-omitted.txt`.

```
thread_child process=2344558 launcher_tid=2344587 git_pids=[2344588, 2344589, 2344590, 2344591] git_rss=18546688 own_rss=7073792 observed_tree_rss=7073792 leader_children=""
thread 'tree_rss_includes_git_children_launched_from_a_nonleader_thread' panicked at tests/records_measure.rs:319:5:
tree_rss 7073792 must include the 18546688-byte Git children [2344588, 2344589, 2344590, 2344591] launched by nonleader TID 2344587; own process RSS is 7073792
```

The observed whole-tree value equals the test process's own RSS to the byte while 18,546,688 bytes of live Git children exist under a nonleader TID and the leader children list is empty. The assertion therefore fails on the actual omission, not on an import, type, or fixture error. The four Git children are real `git cat-file --batch` processes blocked reading their piped stdin, each with an independently known `Child::id`; the launcher holds them until the guard drops, which kills and reaps them on the panic path as well.
Four children are held rather than one because a small Git process holds little resident memory, and the omitted memory must clearly exceed ordinary process-RSS drift for the assertion to be decisive in both directions.

## GREEN — the corrected observer

Command: `/home/abdwhb/.cargo/bin/cargo test --frozen --test records_measure -- --exact tree_rss_includes_git_children_launched_from_a_nonleader_thread --nocapture`. Exit 0. Raw output: `green-01-thread-child-observed.txt`.

```
thread_child process=2356777 launcher_tid=2356806 git_pids=[2356807, 2356808, 2356809, 2356810] git_rss=13115392 own_rss=7028736 observed_tree_rss=20770816 observed_members=[(2356777, None), (2356807, Some(2356806)), (2356808, Some(2356806)), (2356809, Some(2356806)), (2356810, Some(2356806))] leader_children=""
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.04s
```

The corrected observer reports every child under the nonleader TID that launched it, and the summed RSS (20,770,816) exceeds own RSS plus the independently read child RSS (7,028,736 + 13,115,392 = 20,144,128). The test revision used for RED asserted only this numeric coverage because the membership relation did not exist yet; the published revision additionally asserts the reported membership pairs `(child, Some(launcher_tid))`, which strengthens the same claim rather than replacing it. `rustfmt` also reflowed the file before the published hash was taken, so the RED-stage source is not byte-recoverable; the pre-packet source and the published source are both preserved.

## Real-capture coverage

The unchanged real capture fixture was re-run with the corrected observer (`measure-current.txt`). Each combined capture+save journey reports the Git children it actually observed:

```
combined case=one_file ... tree_rss_peak=34279424 git_children_peak=1 git_via_leader_peak=0 git_via_other_thread_peak=1 git_child_listed_under=Some((2382930, 2372415)) process_pid=2372414
combined case=five_file ... tree_rss_peak=34279424 git_children_peak=1 git_via_leader_peak=0 git_via_other_thread_peak=1 git_child_listed_under=Some((2383024, 2372415)) process_pid=2372414
combined case=large_batch ... tree_rss_peak=52887552 git_children_peak=1 git_via_leader_peak=0 git_via_other_thread_peak=1 git_child_listed_under=Some((2383089, 2372415)) process_pid=2372414
```

Git PID 2382930 is listed only under TID 2372415 of process 2372414, so the real capture's Git children are thread-launched exactly as the final-r1 review observed, and the corrected observer counts them. `git_via_leader_peak=0` on all three journeys records that the old reader would have missed every one of them; `tree_rss_peak` rises from the save-only 22–24 MB band to 34.3 MB (one/five file) and 52.9 MB (large batch) because the capture's Git children are now included. The protocol asserts `git_children_peak > 0` per combined journey so an unobserved child cannot be reported as a complete measurement.

## Correction shape

`owned_tree_pids` walks every `/proc/<pid>/task/<tid>/children` list for each owned process, recurses, and deduplicates by process id; it returns each discovered process with the TID that listed it. `tree_rss` sums `VmRSS` once per distinct process id over that set. No process is counted as another address space, no dependency or supervisor was added, and the observer remains the single implementation used by the protocol, the peak helpers and the regression test. The two tests share a mutex so the blocked four-child fixture can never be resident during the normal workload measurements.

## Limits

Sampling remains 1 ms, so a child that is born and exits between samples is still missed; the observation is not proof of every instantaneous peak. Per-process RSS is summed across processes, so pages shared between the test process and a child (for example shared libraries) are counted more than once, which biases the tree total upward. A process that dies between being listed and being read contributes nothing, and that death race is reported as a sampling limit rather than as unavailable evidence. Process ids can in principle be reused inside one sample window; the window here is a few milliseconds and no reuse was observed. This is one warm-cache host, one load snapshot and a debug-profile build, and it establishes no ADR-007 future product budget.
