# Admission formula and measured storage — P02

## Settings in force

- SQLite WAL mode with `synchronous = FULL`, set and read back on every open.
- Busy timeout 2 s; admission and writes occur inside `BEGIN IMMEDIATE`.
- Page size read from `PRAGMA page_size` (4096 on this build); bundled default
  reserved bytes are zero.
- Auto-checkpoint at SQLite's default of about 1000 WAL pages, run by the
  committing thread. The WAL file is reused, not normally truncated
  (`journal_size_limit` remains unset).
- `-shm` and `-wal` are counted while they exist; the last connection close
  removes them, so measured usage can legitimately drop after reopening.

Official sources used (retrieved 2026-11-17):

- <https://sqlite.org/wal.html> — automatic checkpoint at about 1000 pages,
  checkpoint and reset run at commit, WAL file is recycled and not normally
  truncated, sync on every commit with `FULL`.
- <https://sqlite.org/fileformat2.html#overflow> — overflow page carries
  `usable size - 4` payload bytes after its 4-byte next-page pointer.
- <https://sqlite.org/fileformat2.html#walformat> — 32-byte WAL header and a
  24-byte frame header followed by one page.
- <https://sqlite.org/pragma.html#pragma_synchronous> — `FULL` is ACID in WAL
  mode; application crashes remain durable; power loss depends on the VFS.
- <https://sqlite.org/pragma.html#pragma_journal_size_limit> — limits the file
  left after a transaction/checkpoint; it is not a peak-growth bound.

## Formula

For an admitted capture:

```text
payload      = fixed capture metadata
             + Σ(change path + 256)
             + Σ(source path + generation + 512 + source bytes)
             + configuration bytes

overflow     = ceil(payload / (page_size - 4))
row_pages    = (changes + sources + 1) * 8
pages        = overflow + row_pages + 16

wal_bound    = pages * (page_size + 24) + 32
reservation  = 2 * wal_bound + current_wal_file_size + 64 KiB
required     = retained + reservation
admit only when required <= limit
```

Rationale:

- Every payload byte is charged as if it needed an overflow page; every row
  gets eight b-tree pages; 16 pages cover header/freelist/rounding. This is
  deliberately above a best-case page packing.
- The WAL bound includes the 24-byte frame header and 32-byte WAL header.
- The factor two covers a checkpoint that leaves the same committed content in
  both the WAL and the database during the commit/checkpoint path.
- `current_wal_file_size` is added because an auto-checkpoint can backfill
  already committed frames into the database while the WAL file remains; that
  backfill is additional DB growth not caused by the new capture.
- 64 KiB covers `-shm`, WAL headers and file-system rounding.
- `retained` is the actual sum of the database plus existing `-wal`, `-shm`
  and `-journal` file sizes. Free pages reduce actual growth, so the formula is
  conservative, never an exact prediction.

This is application admission, not a filesystem quota. It does not reserve
disk blocks, detect a full device in advance, or bound other processes' files.
A write that fails on I/O remains an explicit error; SQLite's transaction
atomicity prevents a partial record.

## Measured near-boundary write

Fixture: eight untracked files of exactly 1,048,576 bytes (`x`), one committed
configuration file of 186 bytes; `Local` capture.

| Fact | Bytes |
| --- | ---: |
| Capture retained payload (`stats().retained_bytes`) | 8,388,794 |
| Retained store before admission | 65,736 |
| Current WAL at admission | 45,352 |
| Overflow pages (÷4092) | 2,051 |
| Row pages (17 rows × 8) | 136 |
| Paged bound (incl. 16) | 2,203 |
| WAL bound (2,203 × 4,120 + 32) | 9,076,392 |
| Reservation (2 × WAL + WAL backlog + 64 KiB) | 18,263,672 |
| Reported `required` | 18,329,408 |
| Actual final usage while open | 16,941,328 |
| Sampled peak during the save | 16,752,912 |
| Peak samples (1 ms interval) | 140 |

The 8 MiB payload produced 16.9 MiB of final database-plus-WAL storage because
the data exists in both the checkpointed database and the recycled WAL; a sum
of source lengths alone would have under-admitted it. The sampled peak stayed
below the reported requirement, and the final retained size did too.

Sampling limits: the monitor reads file metadata every millisecond from another
thread; an instantaneous peak between samples is not observed, so the recorded
peak is a lower bound. The writer's own `BEGIN IMMEDIATE` serializes other
writers during the measured save.

## Repeated writes and checkpoints

Six successive 1 MiB captures in one open store with a 64 MiB limit. Retained
usage after each save:

```text
1,149,296
2,232,856
3,316,416
8,635,288   <- auto-checkpoint around the 1000-page threshold
8,635,288
8,635,288
```

The jump shows checkpoint duplication: the database grew while the recycled
WAL file remained allocated. Later writes reused the WAL file, so usage stayed
flat. All six records remained readable, and no write exceeded the limit.

A dedicated backlog test confirms the `current_wal_file_size` term: with about
3 MiB of committed but uncheckpointed WAL, the same tiny capture's reservation
was larger by exactly the WAL size before close, compared with the reopened
store after the last connection had removed the WAL.

## Boundary and refusal behavior

- A budget of zero opens the store and reports retained data; new saves return
  `BudgetExceeded { retained, limit, required }`.
- `required - 1` refuses and `required` admits the same capture; the admitted
  post-save usage is at or below `required`.
- Refusal writes no capture/change/source row; prior records remain readable.
- A store whose retained data already exceeds the limit opens and reads
  normally; only new collection is refused.
- Two independent processes with a limit that admits one capture: exactly one
  saved, the other was refused with the budget reason; the baseline record
  remained readable.
- A writer killed while blocked before commit left no record; the same capture
  was then admitted under the same limit, showing no durable reservation leak.
- Replacing the main database path with a directory made accounting return
  `Unavailable`; the save was refused rather than admitted unbounded.

## Overshoot and failure handling

If the formula were to under-estimate, the record is already committed and is
not deleted; `usage()` reports the actual retained size and warning state, and
subsequent saves refuse once `required` exceeds the limit. No automatic
evidence deletion or silent correction occurs. I/O or database failures return
explicit errors; the all-or-nothing transaction leaves no partial record.

The formula does not protect against a caller choosing a huge budget, another
process writing the same database through a different path spelling, or a
filesystem that reports sizes differently from its actual allocated blocks.
