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
the audit continuation was committed and pushed as `dea2cd1d`. A later
user-directed audit follow-up began from P6 T1 revision `80a889f0` and is
recorded in the walkthrough and C1 checkpoint. The C1 checkpoint remains the
shared authority.

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

The user-directed audit follow-up corrected the String/Substring adapter and
all identified `Option<Int>` byte consumers. It also corrected the stale Rust
Map contract assertion. On that follow-up tree, `core_collections` passes 13/13;
`modules_parse_package` selects four tests and passes three, with 11/12 runtime
fixtures passing. The remaining failure is the associated-family invocation
lowering path in the token-backed Cursor assertions in `source-test.ph`. The
exact individual assertion is not isolated; see [the consultation packet](COLL006.C1.P5-G6B-associated-family-invocation-consultation.md).

## Retained residuals / P9 obligation

- R-PARSE-TEXT: corrected in the user-directed follow-up by materializing the
  returned Substring to String at the adapter boundary.
- R-PARSE-BYTES: corrected in the user-directed follow-up with explicit
  `Option<Int>` handling in parser and operator-fixture consumers.
- R-PARSE-ASSOCIATED-INVOKE: full-package execution remains blocked when the
  token-backed Cursor assertions invoke `current`/`advance` for `Option<Token>`;
  the compiler reports associated-family invocation lowering is not implemented
  until associated semantic resolution lands. See the consultation packet and
  preserve this as an owning compiler-layer issue.
- R-MAP-REMOVE: the implementation already returns the Map from public
  `remove(_)`. The old Rust assertions expected the internal `rawRemove`
  Option result and were corrected. `core_collections` now passes 13/13; remove
  this from active failures while retaining the earlier baseline observation as
  history in the checkpoint.
- Global format drift and P4 D1/D2/D4 remain inherited verification residuals.

The updated complete package gate passes its declared project compile and the
independent snapshot/owner tests, but remains blocked by the associated-family
invocation failure above. P9 must rerun
`RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package`
after the owning compiler work and classify any surviving failures against
contemporaneous main. G6.A never waives G6.B. Do not claim full parser-package
certification until the complete command passes.

Next: continue P6 only after the public API contract consultation returns its
decision. Do not absorb the associated-family compiler issue or additional
parser changes into P6. Broad tests, Clippy, release certification, and final
residual classification remain P9 responsibilities.
