# Records design — P01

## Public interface

`anti_slop::records` exposes one deep module and no repository, service, factory or migration layer:

```text
Store::open(&Path) -> Result<Store, StoreError>
Store::save_capture(&mut self, &Capture) -> Result<CaptureId, StoreError>
Store::read_capture(&self, CaptureId) -> Result<RetainedCapture, StoreError>
RetainedCapture::{id, selection, resolved_commit, generation, configuration, changes, context, slice}
RetainedChange { path, before, after }
RetainedSource { path, role, generation, git_blob, git_mode, bytes() }
RetainedConfiguration::{generation, source, captured_bytes}
```

`save_capture` consumes the existing `workspace::Capture` read-only. It does not re-run acquisition, path selection or configuration resolution; it snapshots the already-compiled capture. `Role`, `GitMode`, `Selection` and `SourceRange` keep their `workspace` meaning and are reused rather than copied into parallel record types. `config::Source` remains the configuration-source meaning.

`RetainedCapture::slice` mirrors `Capture::slice`: a range resolves only when path, role and generation all bind the same retained source, so attribution cannot drift to another view of the same path.

`StoreError` keeps SQLite types behind the boundary and separates the states the caller must distinguish: unavailable storage, unsupported newer schema, not a capture store, busy writer, damaged store, failed operation, missing record and unbound range. SQLite message text is carried as detail; `rusqlite` types are not.

## Schema

Schema version 1 is stored in `PRAGMA user_version` and is the only supported version. Three tables carry the record:

- `captures`: one row per save occurrence, with `AUTOINCREMENT` identity, comparison selection (kind plus optional base reference), resolved commit, capture generation, and the complete configuration identity (generation, built-in/file kind, file path, SHA-256, size and captured bytes).
- `capture_changes`: ordered change paths, preserving the workspace's rename-attribution grouping. The `Change` path is stored separately from the source paths, because a rename's change path differs from its before path.
- `capture_sources`: ordered before/after/context sources with path, role, generation, Git blob, Git mode and exact bytes. `change_ordinal` groups a change's sides and is `NULL` only for context rows; a table check enforces that pairing.

Checks enforce the representable states: a base selection has a reference and other selections do not; built-in configuration has no file columns and file configuration has all of them; context has no change ordinal; roles and modes are known values. Foreign keys are enabled and every child row references its capture. There is no external blob file, no separate evidence table and no generic record framework.

Every source row and the capture row are inserted inside one `BEGIN IMMEDIATE` transaction and committed together. All bytes are BLOBs in that transaction. No acknowledgement exists before `commit()` returns.

Why these choices:

- **One row per save, not per content hash.** Repeated saves and repeated equal source bytes are separate occurrences with their own identities. No deduplication merges provenance. `AUTOINCREMENT` prevents a future deletion from reusing an identity.
- **Generation strings retained as observed.** `workspace` owns their computation; `records` neither recomputes nor normalizes them. That keeps the capture's generation semantics in one owner.
- **Configuration bytes retained.** ADR-005 requires actual evidence rather than a hash alone; the compiled keys and origin pointers can be regenerated from the bytes, so no second rendered copy of the configuration view is stored.
- **No migration framework.** No earlier supported stored schema exists, so a migration path would be invented. A newer version is refused; an unrelated or inconsistent store is refused before mutation.
- **No purge, export or analysis fields.** They are excluded by the packet and ADR-005 owns their later semantics.

## Failure and ambiguity semantics

- A failed insert, failed commit or aborted transaction rolls back the whole capture. The injected-trigger test verifies unchanged row counts and that the earlier capture still reads.
- A caller never sees `CaptureId` before commit. A `Busy` result after the bounded wait leaves no row.
- If the process dies after commit but before the caller records the acknowledgement, the record is present. The API does not convert that into failure, and the tests assert the record survives a killed writer. Callers must not infer absence from a lost acknowledgement or from an error race around it.
- Opening a store on missing storage, a non-database file or an unrelated database returns an explicit error. Opening never silently adopts an existing non-empty database with `user_version = 0`.
- Initialization is one DDL transaction. A conflicting pre-existing schema object makes creation fail and leaves `user_version = 0` with no created tables.

## Concurrency

`BUSY_TIMEOUT = 2 s` is set on every connection. `BEGIN IMMEDIATE` acquires the write lock before the first insert, so a competing writer either waits and then commits, or returns `Busy` after the bound. WAL keeps retained reads available while a writer holds its transaction. The tested contention path is one blocked save: it waits about the bound, stores nothing, and succeeds after the lock is released.

Budget admission, reservations that stop two writers from admitting the same remaining budget, and threshold refusal are explicitly not implemented in this packet.

## Deliberate non-changes

`src/config/` and `src/workspace/` were not modified. The capture already exposes retained generations, roles, modes, blobs, ranges and configuration provenance through public accessors. Correcting or widening those types was unnecessary and would have risked acquisition or identity semantics outside this packet's objective.

The module has no background writer, queue, shutdown protocol, cached statements or extension points. The packet's outcome does not need them, and adding them would create lifecycle state without a demonstrated producer.
