# Demonstration intake — durable capture, fresh-process read, quota boundary

## What the viewer should be able to judge

A library consumer can capture a real selected change plus separate context,
save it, lose the original repository, and read the exact retained bytes and
provenance from a fresh process. The same run shows a quota refusal that leaves
the prior record readable. There is no product CLI for records and no UI; the
evidence is sanitized command output from the production library boundary.

## Scene and actors

- Actor: a Rust application using `anti_slop::records::Store`.
- Starting state: a temporary Git repository with committed `a.ts` and
  `guide.ts`, a committed `.anti-slop.jsonc`, a selected working change in
  `a.ts`, and `guide.ts` supplied as context.
- Action: capture, save with a 4 MiB budget, attempt a second save under a zero
  budget, delete the repository, read the first record in a child process.
- Observable result: exact hex bytes/roles/generations after the origins are
  gone, saved footprint, and an explicit quota refusal with the prior record
  still readable.

## Runnable local trial

```sh
cargo test --frozen --test records_demo -- --nocapture --test-threads=1
```

Observed output (sanitized; retained in `demonstration-output.txt`):

```text
capture selection=Local commit=2926a2173ff66fae78b822525467d5634ca164b8
        generation=ee0d5c569435ac5f9f25fde2cf0e7d1f050122d28104e57ba9521d3150f1d732
selected path=a.ts after_role=Working
before=686561640a after=776f726b696e672073656c65637465640a context=67756964650a
saved id=1 retained=90456 limit=4194304 warning=false over_limit=false
quota refused retained=90456 required=687568 while prior id=1 stays readable
origins removed: true
fresh process read:
id=1 selection=Local commit=2926a2173ff66fae78b822525467d5634ca164b8
   generation=ee0d5c569435ac5f9f25fde2cf0e7d1f050122d28104e57ba9521d3150f1d732
path=a.ts role=Working bytes=776f726b696e672073656c65637465640a
   generation=e9e019435c08bbbbb8a121fddb481c3fa3a9cbbcd70d3e4859d3401df29cbbcf
context_role=Context context_bytes=67756964650a
```

The hex values decode to `head\n` (before), `working selected\n` (selected
after), and `guide\n` (context). The fresh process opens the database by path,
reads the record by id, and does not touch the removed repository.

## Applicability and limits

- Repository: `anti-slop`, base `45dac3c241338c94166369fe821fe1d3499cf974`,
  with the uncommitted AS-M004 source delta described in the P03 report. The
  demonstration is bound to that dirty tree, not to a published head.
- Environment: Linux, bundled SQLite 3.54.1, WAL with `synchronous = FULL`,
  warm OS caches; the temporary fixture is created by the test.
- Test-target overhead and the test harness are included; this is not a CLI
  latency claim and not a hosted trial.
- The quota refusal is a synthetic zero budget, not a user-configured limit;
  the 4 MiB budget is also below the 10 GiB default to keep the scene small.
- No screenshots are produced because there is no visual surface.
- The PR body/demonstration publication remains with Firstmate; this file and
  its linked evidence are the review-ready source material.
