---
id: COLL006.C1.P5
program: COLL006
checkpoint: COLL006.C1
kind: handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P5 — Final handoff to C1 follow-up work

P5 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. The current implementation
anchor is `main` at `6057da95150ceb13796ff183b19c31207590d912`; the later
documentation closeout records the final package-gate status. The canonical C1
checkpoint remains the shared state authority. P5 owns no remaining parser
package blocker.

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
- At P5 implementation closure, the native surface was 429 declarations
  (417 + 12); Value/Object remained 16/48 bytes. Later C1 additions change the
  overall native count. Preserve P1–P4 semantic/static/runtime identities and
  ownership.

## Accepted evidence

G1–G5/G7, P5-owned T9, and both G6 subgates are recorded in the
checkpoint/walkthrough. G6 is closed:

```yaml
T7: IMPLEMENTED / FOCUSED_TESTED
G6.A: PASS
G6.B: PASS
aggregate_G6: PASS
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

The current package result is 4/4 Rust integration checks and 13/13 runtime
fixtures. It includes the typed Mini-TOML composition and token-backed Cursor
path. The original failure was missing semantic owner context: `Some(...)` in
an equality expression supplied no expected `Option<Token>` owner, so its
nested constructor could not resolve. The fixture now uses
`Option<Token>::Some(...)` and structural variant matching. The existing static
constructor route and semantic-lowering contract are unchanged.

## Separate follow-up and verification boundary

- Dynamic associated-family invocation
  (`AssociatedResolutionKind::DynamicInvoke` →
  `AssociatedLoweringSpec::DynamicInvoke` → `compile_associated_invoke`)
  remains unsupported. It is separate from the statically owned
  `Option<Token>::Some` construction proven by G6.B and requires its own plan
  if pursued.
- Contextual `option == Some(value)` inference could be considered separately
  as an owner-hint feature. It must not change heterogeneous equality rules.
- Workspace tests, Clippy, and release certification were not run. P5 is
  `FOCUSED_TESTED`, not `RELEASE_COMPLETE`.
- P8 T8/G8 and the broader P9 convergence/certification work remain active in
  C1. Consult the C1 checkpoint for their current status; this handoff does not
  close the full C1 checkpoint.

## Must-read files for later C1 work

- `../COLL006.C1-CHECKPOINT.md` — canonical plan, verification, and residual state.
- `COLL006.C1.P5-detailed-implementation-plan.md` — P5 scope and G6 closure.
- `COLL006.C1.P5-walkthrough.md` — implementation and evidence.
- [P9 residual ledger](../P9/COLL006.C1.P9-residual-ledger.md) — remaining
  P8/C1 residuals.
- [Parse package README](../../../../../packages/parse/README.md) — package
  architecture and current limitations.

## Recommended first checks

Use the declared package gate only when a later change affects parse package
behavior:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package
```

Do not re-open P5's static constructor path as an assumed generic lowering
defect. Preserve semantic resolution → canonical `AssociatedResolution` →
semantic-lowering projection → `AssociatedLoweringSpec` → compiler emission →
existing VM construction/invocation machinery.

## Resolved parse and test-oracle residuals

- R-PARSE-TEXT: corrected in the user-directed follow-up by materializing the
  returned Substring to String at the adapter boundary.
- R-PARSE-BYTES: corrected in the user-directed follow-up with explicit
  `Option<Int>` handling in parser and operator-fixture consumers.
- R-PARSE-ASSOCIATED-INVOKE: resolved for the P5 fixture and declared package
  gate by supplying the exact `Option<Token>` owner. Dynamic associated-family
  invocation remains a distinct capability gap.
- R-MAP-REMOVE: the implementation already returns the Map from public
  `remove(_)`. The old Rust assertions expected the internal `rawRemove`
  Option result and were corrected. `core_collections` now passes 13/13; remove
  this from active failures while retaining the earlier baseline observation as
  history in the checkpoint.
- Global format drift and P4 D1/D2/D4 remain wider C1 verification concerns;
  they do not block the completed parse-package gate.

```yaml
packages_parse:
  implementation: COMPLETE
  focused_verification: PASS
  package_integration: PASS
  workspace_verification: NOT_RUN
  release_certification: NOT_RUN
```
