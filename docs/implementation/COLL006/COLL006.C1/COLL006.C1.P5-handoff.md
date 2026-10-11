---
id: COLL006.C1.P5
program: COLL006
checkpoint: COLL006.C1
kind: handoff
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
---

# COLL006.C1.P5 — Handoff to P6/P9

P5 is COMPLETE / IMPLEMENTED / BASELINE_BLOCKED on feat/telos-collection-upgrade
in `.worktrees/telos-collections-upgrade`. Bulk implementation is `f3403d70`;
the audit continuation is uncommitted. C1 checkpoint remains the shared authority.

## Stable takeover interfaces

- Canonical unary Slice/Buffer types inherit Collection; no new literal family,
  static value, materialization kind, or inference engine exists.
- `heap/slice.rs`: strong normalized owner/start/len; Array/Vector/Buffer sources;
  Slice subviews flatten to root. The tracer retains owner, owner traces elements.
- `heap/buffer.rs`: logical capacity separate from Vec capacity, initialized
  prefix only, fallible reservation, atomic full push, initialized-only set,
  no implicit growth/shrink. Successful pushes reuse the reservation.
- Array, Vector, Buffer, and Slice source APIs use canonical Range.rawSliceBounds.
  Slice is read-only; live Array/Buffer writes remain visible without view resize.
- Slice and Buffer are rejected as Map/Set keys; Vector remains key-safe.
- `ListSource<T>` has one Slice field; `<T>[*items]` creates the private Array.
  Snapshot membership/order/references are independent of caller List mutation.
- Native surface is current at 429 declarations (417 + 12). Value/Object are
  16/48 bytes. Preserve P1–P4 semantic/static/runtime identities and ownership.

## Accepted evidence

G1–G5/G7 and P5-owned T9 evidence are recorded in the checkpoint/walkthrough.
User-approved audit disposition 2 amends G6:

```yaml
T7: IMPLEMENTED / FOCUSED_TESTED
G6.A: PASS
G6.B: BASELINE_BLOCKED
aggregate_G6: BASELINE_BLOCKED
```

Standalone CLI fixture command:

```sh
RUSTFLAGS='' cargo run -p phalcom-core --bin phalcom -- packages/parse/tests/runtime/src/core/list-source-snapshot-test.ph
```

Focused Rust selector (2 actual tests):

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core parse_list_source_snapshot -- --nocapture
```

Both pass. The native test constructs a real ListSource, inspects Slice/Array
storage, collects the caller List with only source rooted, and reads/parses the
retained heap tokens after forced GC. Do not replace it with generic heap tests.

## Retained residuals / P9 obligation

- R-PARSE-TEXT: TextSource.slice promises String but String.slice returns
  Substring. The audit recommends initial preservation of the String promise,
  materializing views in a bounded P16 parser-migration follow-up.
- R-PARSE-BYTES: rawByteAt/String.bytes.at return Option<Int>; TextSource,
  parse.text, and operator-fixture consumers treat bytes as Int. Include at,
  lineColAt, optional presence handling, and String+Substring fixture use in that
  follow-up. The comparator is not the established root cause.
- R-MAP-REMOVE: collection-contract absent-key removal assertion fails identically
  on pre-P5 `78cfd920`. Keep unrelated; P9 applies the final main-comparison policy.
- Global format drift and P4 D1/D2/D4 remain inherited verification residuals.

The exact complete package gate was rerun: 2 Rust passes/2 failures, 11 of 12
runtime fixtures fail at the inherited sites. Keep its true failure status.

P9 must recheck both parser residuals, verify owning corrective work, rerun
`RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package`,
compare surviving failures with contemporaneous main, and classify each under
the checkpoint policy. The current parser provenance is PRE_P5_BASELINE with
audit-confirmed unchanged source compatibility; main runtime reproduction is
not verified. G6.A never waives G6.B. Do not claim full parser-package certification
until the complete command passes.

Next: plan P6 ordered-hash/associative collections using established collection
storage and key boundaries. Do not absorb parser/text baseline repairs into it.
Broad tests, Clippy, release certification, and final residual classification
remain P9 responsibilities.
