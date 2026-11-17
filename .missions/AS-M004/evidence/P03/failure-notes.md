# Controlled failure observations — P03

The real public capture/store boundary runs in a fresh child process with a
small Linux `LD_PRELOAD` interposer built by the existing C compiler
(`tests/support/records/io_fault.c`). The interposer is test-only and is not
linked into the library. It is enabled only after a baseline capture commits, so
the fault applies to the attempted new capture.

## Commit I/O failure

`commit_io_failure_keeps_acknowledged_records_and_no_partial_capture`:
mode `write` fails the next `pwrite`/`write` to the real `-wal` file with
`EIO`. The child reports:

```text
controlled commit failure: error:Capture store is unavailable: disk I/O error
```

After the child exits, the database has one committed capture, one change and
one source; the baseline bytes remain readable through a reopened store. The
attempted capture is absent. No partial record was observed.

This is a real SQLite WAL write failure after the transaction began. It is not
a simulation of a device or power failure, and it does not prove that an
arbitrary filesystem honours or fails `fsync` in a particular way. The existing
pre-commit lock-wait kill remains a separate case; it is not relabelled as an
in-commit kill.

## Process killed inside commit at the WAL sync seam

`process_killed_during_commit_is_complete_or_absent`: mode `pause` raises
`SIGSTOP` inside `fdatasync` for the WAL **before** the real sync returns. The
interposer writes a parent-visible ready marker first. The child has already
written the attempt's WAL frames; the parent kills it with `SIGKILL` at that
point and reopens the real store.

Observed outcome: `complete; counts=(2, 3, 3)`. The baseline capture is present
with its exact bytes; the attempted capture (id 2) is present with two exact
changes and passes the retained integrity read. The total rows are consistent:
two captures, one baseline change plus two attempt changes, and three sources.

The capture is tiny (one added file), so it does not reach SQLite's cache-spill
path; the dirty pages and commit record are written as part of commit and the
child is stopped at the WAL synchronization call inside that commit. This is a
real process interruption during commit, not the lock-wait kill and not the EIO
return.

What it proves: after a process is killed inside commit, the next real open
observes either an absent capture or a fully valid one, never a partial row
set; here the committed bytes survived in the OS page cache and recovery
accepted the commit. What it does not prove: power-loss durability. The real
`fdatasync` never returned, so a power loss could still drop data that a process
kill leaves visible. The complete-or-absent/ambiguity result is the contract;
this is not a physical durability certification.

## Postcommit measurement failure

`postcommit_measurement_failure_preserves_the_committed_record`: mode
`unmeasure` renames the database main file after a WAL `fdatasync` and before
the caller's post-commit measurement. The child receives:

```text
postcommit measurement failure: error:Capture 2 was committed, but its retained
storage could not be measured: Capture store is unavailable: Cannot measure the
capture store: No such file or directory (os error 2)
```

The parent restores the renamed database, reopens it, and reads capture 2 and
its `attempt` bytes. A failed measurement therefore does not claim the record
was absent and does not lose the committed truth.

## Postcommit over-limit outcome

`postcommit_over_limit_is_reported_with_committed_identity`: mode `over`
creates an extra 2 MiB `-journal` file after a successful WAL `fdatasync`. With a
1.5 MB admission budget, the child's returned result is:

```text
postcommit over-limit result: saved:2:over=true:retained=2212328
```

The record is committed and readable. Opening the store later may let SQLite
remove the synthetic `-journal`; the child's post-commit result is the
authoritative observation that the committed footprint crossed the budget. The
record is not deleted to restore the quota.

## Covered here versus still absent

The suite covers unavailable storage and malformed/newer schema, atomic
initialization failure, missing/altered/duplicate/extra stored evidence,
transaction rollback, bounded writer contention, a writer killed after commit
before acknowledgement, and a writer killed while waiting before commit. P03
adds `read_only_database_storage_fails_explicitly` (read-only database file and
directory produce an explicit `Unavailable`), the actual commit write failure,
a process killed inside commit at the WAL sync seam, and both postcommit
outcomes.

Not simulated: hardware or power loss, a lying device `fsync`, filesystem
remount/read-only transition during a live transaction, or an actual SQLite
journal I/O failure for every journal mode. The interposer proves process-level
error handling at the SQLite call boundary, not physical media behaviour.
