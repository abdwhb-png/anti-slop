# Retained-capture integrity — P01C1

## Defect and correction boundary

The rejected P01 candidate returned a saved capture as complete after a retained
source row was deleted, its bytes were changed, or its provenance was changed.
The read path reconstructed whatever rows remained and returned success.

This correction adds one records-owned integrity digest over the complete
retained membership, provenance and bytes. It is written in the same SQLite
transaction as the capture and verified before `read_capture` returns any value.

## Mechanism

At save:

1. The canonical retained rows are derived from the `workspace::Capture` before
   insertion: change ordinals and paths, then source ordinals and
   `(change_ordinal, side, path, role, generation, git_blob, git_mode, bytes)`
   in insertion order, plus the comparison and configuration identity already
   retained in the capture row.
2. A domain-separated SHA-256 (`anti-slop.records.integrity.v1`) is computed
   over that canonical data with length-prefixed fields and explicit presence
   markers for optional values.
3. The digest is stored in `captures.integrity` inside the same
   `BEGIN IMMEDIATE` transaction as the capture, changes, sources and bytes.

At read:

1. The header and every allocated change/source row are read in one snapshot.
2. The same digest is recomputed from the stored values, including row ordinals.
3. A mismatch returns `StoreError::Corrupt("Retained capture failed its
   integrity check")` before any `RetainedCapture` is constructed.
4. Reconstruction then rejects an unknown change ordinal, an inconsistent
   side/change pairing, or a second before/after source for one change.

The digest is not a new workspace or configuration identity. It does not
recompute `Capture::generation`, source generations or configuration
generations; those values are retained and hashed as opaque evidence. The
workspace and config modules keep sole ownership of their identity algorithms.

## What it detects

With one digest written and verified per capture, a read rejects:

- a missing source row (the source sequence no longer matches the retained
  membership);
- a missing change row (its sources refer to an unknown ordinal, and the
  change list no longer matches the digest);
- altered source bytes, including same-length changes;
- altered configuration bytes, including same-length changes;
- altered role, generation, Git blob, Git mode, side, path or change ordinal;
- altered comparison/config/capture identity fields in the capture row;
- duplicate or extra sources, extra changes, extra context rows, and reordered
  ordinals;
- a store written by the pre-correction schema (its missing `integrity` column
  is rejected at open as `NotACaptureStore`).

The repair is not a migration: schema version 1 was never shipped, so the
column is added in place and no historical schema is claimed.

## What it does not prove

- It does not protect against a party that rewrites the evidence **and** the
  digest consistently, or replaces the whole database with a self-consistent
  forged store. There is no keyed MAC or external trust anchor.
- It does not detect deletion of the whole capture row: that is reported as
  `NotFound`, not as integrity failure.
- It does not prove the physical integrity of the storage device or filesystem.
  SQLite may report a malformed database, I/O failure or corruption first; the
  digest catches a logically inconsistent record that SQLite still serves.
  Bit rot that coincidentally edits both data and digest is out of scope.
- It does not recover or repair bytes, and it does not make a stale workspace
  generation fresh. Workspace generation identity and analysis confidence are
  separate concerns.
- The digest adds one SHA-256 pass over retained bytes at save and read. No
  measured latency, memory or growth claim is made here; resource qualification
  remains later mission work.

## Transaction and failure behaviour preserved

- The digest, capture row, changes, sources and bytes commit in one transaction.
- A failed insert or commit still rolls back the entire capture; no digest is
  visible for an uncommitted record.
- A lost acknowledgement after commit still leaves a fully readable, verified
  record; absence is not inferred.
- Deletion-versus-empty and valid empty captures keep their previous meaning:
  presence is digested explicitly and empty byte sequences are length-prefixed,
  so `None` and `Some(b"")` remain distinguishable.

## Dependency cleanup

`rusqlite` was removed from `[dev-dependencies]`; the single qualified exact
dependency remains under `[dependencies]`, and integration tests use it from
there. No package or version was added, removed or changed in `Cargo.lock`.
