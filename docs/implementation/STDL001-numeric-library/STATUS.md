---
id: STDL001
status: BLOCKED
completion: PARTIAL
verification: UNVERIFIED
---

# STDL001 status

The numeric library has completed foundational planning records and several
proposed implementation records; release verification remains open. Source
acceptance for the active Rational, statistics, and probability implementations
remains blocked by canonical Universe bootstrap. The architect's carrier/frame
correction passes the 64-coefficient Float Horner semantic regression on 2 MiB.
The minimal project compiles, then `VM::new()` rejects associated invocation
lowering in `universe:callable.function`; this current failure is distinct from
the historical semantic stack overflow. Clean-main `c2dd635e` completes the same
dependency-free project. See the
[incident report](stack-overflow-incident-2026-10-11.md).

The `STDL001.C6` Rational source slice is present but unverified; see
[`STDL001.C6`](C6-rational/CHECKPOINT.md).

The `STDL001.C7` descriptive-statistics source is present but unverified; its
checkpoint is blocked pending canonical bootstrap repair. See
[`STDL001.C7`](C7-statistics/CHECKPOINT.md).

The `STDL001.C8` probability source is partial and unverified. Its compatibility
choices and remaining boundaries are in
[`STDL001.C8`](C8-probability/CHECKPOINT.md).

All three active checkpoints remain partial and unverified pending the
missing canonical associated-lowering attachment repair and a passing
minimal-project bootstrap regression. The stack ruling has been received and
implemented; it does not establish source-package acceptance.
