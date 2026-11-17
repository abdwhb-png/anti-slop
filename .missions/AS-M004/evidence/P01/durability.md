# SQLite durability choices — P01

Retrieved from the official SQLite documentation on 2026-11-17. The bundled engine is SQLite 3.54.1 (`libsqlite3-sys 0.39.1`, `sqlite3/sqlite3.h`).

## Actual settings

| Setting | Value | Where |
| --- | --- | --- |
| Journal mode | `WAL`, read back and required to report `wal` | Every `Store::open` |
| Synchronous | `FULL`, read back and required to report `2` | Every `Store::open` |
| Busy timeout | `2000 ms` | Every `Store::open` |
| Write lock acquisition | `BEGIN IMMEDIATE` | `Store::save_capture` |
| `journal_size_limit` | Not set; SQLite default `-1` (no limit) | — |
| Auto-checkpoint | SQLite default (`1000` pages) | — |

The stored version is read first, so an unsupported newer database is refused before any journal-mode change. For a new store the journal and synchronous settings are applied before schema creation; for an existing version they are applied after table verification and before the store is returned.

## Official evidence and interpretation

- `PRAGMA synchronous` (https://sqlite.org/pragma.html#pragma_synchronous):
  - `"FULL is atomic, consistent, isolated, and durable (ACID) in WAL mode"`.
  - `"Transactions are durable across application crashes regardless of the synchronous setting or journal mode."`
  - `FULL` is not necessarily durable across power loss in rollback-journal mode; `EXTRA` adds durability there. In WAL mode `EXTRA` is no different from `FULL`.
- Write-ahead logging (https://sqlite.org/wal.html):
  - `"Writers sync the WAL on every transaction commit if PRAGMA synchronous is set to FULL but omit this sync if PRAGMA synchronous is set to NORMAL."`
  - Automatic checkpoints run at about 1000 WAL pages by default; a long reader delays a checkpoint, so the WAL can grow.
- Atomic commit (https://sqlite.org/atomiccommit.html):
  - Commit atomicity depends on `fsync`/flush behavior; SQLite assumes the OS does what it advertises: `"SQLite assumes that the operating system that it is running on works as advertised."` Broken or lying `fsync` primitives remain outside SQLite's control.
  - A commit either becomes visible with all its pages or survives as a rollback; a power loss cannot leave a partially applied transaction. The guarantee is subject to the filesystem and device honoring the sync contract.
- `PRAGMA journal_size_limit` (https://sqlite.org/pragma.html#pragma_journal_size_limit):
  - It limits the size left in a rollback journal or WAL file after a transaction or checkpoint, truncating an already large file to that size.
  - It is therefore **not** a bound on peak write growth. A large transaction can grow the WAL beyond the limit before the next truncation. This implementation does not set it and does not treat it as a peak bound; peak auxiliary growth must be measured rather than inferred from the setting.
- Busy handler (https://sqlite.org/c3ref/busy_timeout.html):
  - `sqlite3_busy_timeout` installs a handler that sleeps repeatedly `"until at least 'ms' milliseconds of sleeping have accumulated"`; afterward `sqlite3_step()` returns `SQLITE_BUSY`.
  - `rusqlite` exposes this as `Connection::busy_timeout`. The 2 s bound is the implementation's declared contention limit, not a claim that a blocked writer can never observe a different SQLite error.

## What the tests demonstrate

- A committed capture survives `SIGKILL` of the writing process after `save_capture` returned but before the child's acknowledgement was written.
- A writer killed while blocked before commit leaves no capture, change or source row.
- A transaction failure injected by a database trigger leaves previous records readable and writes nothing of the failed capture.
- An unsupported `user_version = 2` is refused; a failed schema initialization leaves `user_version = 0` and no tables.

## What remains unproven

- Power-loss durability on the actual host storage and filesystem was not tested; SQLite's claim is conditional on the VFS and hardware honoring sync.
- Commit interruption exactly inside the SQLite commit critical section was not injected. The packet defers the complete crash-injection matrix.
- Peak WAL/database growth, checkpoint behavior under long readers, and memory cost were not measured. Resource qualification remains later mission work.
