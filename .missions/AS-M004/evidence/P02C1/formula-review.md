# Effective database identity and formula review — P02C1

## Defect and fix

SQLite's Unix VFS resolves symbolic links in every path element before it derives
the auxiliary filenames. The installed binding source
(`libsqlite3-sys 0.39.1`, `sqlite3/sqlite3.c`) implements this in
`unixFullPathname`/`appendOnePathElement`: each component is `lstat`ed and a
symlink is replaced by its target before the effective path is assembled.
P02 measured suffixes of the caller's spelling, so a file alias omitted the
real `-wal`/`-shm`, and a parent-directory alias would have the same problem.

`records` now resolves the effective database path before opening and measuring:

- an existing database is resolved with `fs::canonicalize`;
- a not-yet-created database resolves its parent directory and keeps the final
  name, so SQLite creates the file and auxiliaries there;
- a dangling final symlink is followed up to 32 hops to the path SQLite would
  create;
- failures resolving the path are explicit `Unavailable` errors, never a
  silently smaller measurement.

The public regression covers both boundaries: a file symlink and a directory
symlink to the real parent. Both are opened through the alias, receive a real
budget refusal, are admitted only at the reported requirement, and the returned
post-commit usage equals the canonical database-plus-auxiliary footprint.

Held-reader observations from the final test:

```text
file alias: retained=9,657,096 canonical_before=9,657,096 required=26,289,248
            canonical_after=12,854,216 post_commit=12,854,216
dir  alias: retained=12,854,216 canonical_before=12,854,216 required=32,683,488
            canonical_after=16,051,336 post_commit=16,051,336
```

The RED on the same public boundary reported `retained=4,096` for a canonical
footprint of `6,459,976`, admitted `7,016,016`, and returned success while the
canonical files reached `9,657,096` — the reported bytes were only the 4 KiB
main file. Without a held reader, last-close checkpoint removes the WAL and
masks this undercount; the regression keeps a real read transaction open while
building and admitting the backlog.

## Formula assumptions and constants

Input bounds come from the existing capture owner:
`MAX_FILES = 512`, `MAX_FILE_BYTES = 1 MiB`, `MAX_TOTAL_BYTES = 16 MiB`,
configuration file at most 1 MiB, and normalized paths/base references at most
4,096 bytes. Generation strings are 64 hex characters and Git blobs 40 or 64.

| Term | Justification |
| --- | --- |
| `BASE_METADATA_BYTES = 4 KiB` | Fixed capture-row columns, sizes, integrity hash, SQLite record/cell/page headers and rounding. Dynamic generation, hash, reference and configuration-path lengths are added separately. |
| `CHANGE_METADATA_BYTES = 256` | A change row beyond its path: ordinal, record header, cell pointer and page overhead. |
| `SOURCE_METADATA_BYTES = 512` | A source row beyond its dynamic path/generation: side, role, mode, Git blob (≤64 bytes), record and cell headers. |
| `payload / (page_size - 4)` | Conservative overflow-page charging; SQLite's overflow page uses 4 bytes for the next pointer and the rest for payload. |
| `8 pages per row` | Engineering allowance for leaf, interior and split pages over the bounded ≤512-source/≤512-change datasets. It is not a formal worst-case proof; measured peaks stayed below it. |
| `page_size + 24` per frame, plus 32 | Official WAL frame and header sizes. |
| `× 2` | A checkpoint can leave the same committed content in both the WAL and the main database. |
| `+ current_wal_file_size` | An auto-checkpoint can backfill already committed frames into the database while the WAL file remains; the backlog test shows this term exactly. |
| `+ 64 KiB` | `-shm` (32 KiB), WAL headers and file-size rounding. |

`journal_size_limit` is not used as a peak bound, and a `max_page_count` check
or a sum of source bytes would miss page, metadata, WAL and checkpoint costs.

## Supported runtime conditions

- Linux (including WSL2), bundled SQLite 3.54.1, WAL mode, `synchronous = FULL`,
  default auto-checkpoint at about 1000 pages, one writer transaction at a time.
- Page size read at open (4096 here); the bundled default has zero reserved page
  bytes. A custom build with reserved space would invalidate the
  `page_size - 4` overflow assumption and needs re-qualification.
- Readers may hold snapshots; the WAL backlog term covers the checkpoint
  backfill they can produce. Writers are serialized by `BEGIN IMMEDIATE`.
- File and directory aliases are resolved to the effective database identity;
  the same database must be reachable through one effective path.

Not claimed: a hard filesystem quota, protection from device/filesystem failure,
accounting for unrelated files or SQLite temp files outside the workspace,
atomic guarantees under malicious path replacement, or a formal bound for
arbitrary SQLite builds/filesystems.

## Setup and post-commit truth

Opening and schema initialization are not refused because retained bytes exceed
a low budget; retained reads keep working. The schema and file bytes created at
initialization are included in every later admission measurement, so a budget
below them permits reads and refuses new collection.

`save_capture` now returns `SavedCapture { id, usage }`. The usage is measured
after the commit, so a caller cannot receive a bare id while the committed
footprint silently differs. `StorageUsage.over_limit` reports a committed
footprint above the budget; data is never deleted to restore the quota. If the
post-commit measurement itself fails, `StoreError::CommittedUnmeasured { id,
reason }` names the committed record instead of reporting an ordinary failure.
