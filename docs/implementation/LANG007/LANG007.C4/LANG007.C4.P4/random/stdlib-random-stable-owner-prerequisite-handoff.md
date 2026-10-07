# Random prerequisite handoff

Branch: main. Starting HEAD: `c4db36bbab312cd41a589a34dad55dfbfd4ed456`.
Stable owner repair: IMPLEMENTED / FOCUSED_TESTED, uncommitted.
Parent random plan: BLOCKED at unchanged R0 probe.

Read `stdlib-random-implementation-incident.md`, the user consultation ruling,
and `stdlib-random-luna-patch-grade-implementation-plan.md` before continuing.
The checkpoint owns shared state; existing structural/static work is independent.

Established interfaces:

- `StableCallableOwnerRef::{Declaration,Variant,Conformance}` mirrors semantic
  callable ownership. Conformance carries source `StableImplRef` only.
- `try_declaration` is lexical; `enclosing_declaration` includes an exact case's
  enum owner and returns None for conformance.
- Schema v11 tagged ownership and legacy v10 decoding are regression-tested.
- Generic parameter fingerprints discriminate owner kind and exact source owner.

Do not redesign these interfaces or restore total declaration ownership.
Do not weaken the original probe or introduce exact-specialization identities.

Next decision: authorize bounded evidence-publication/runtime-recipe diagnosis.
Current failure is `missing runtime conformance recipe for semantic slot 0`
at `ProbeUniform.sample(using: sequence)` in `random_syntax.ph`.
Inspect `phalcom-core/src/modules/compile.rs` evidence layout/binding matching
and the semantic specialization conformances feeding it. Do not invent evidence
in the compiler or change witness selection without an explicit decision.

First discriminator:
`RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus random -- --nocapture`.
Then resume the original random DAG once R0 passes. Unrelated dirty user files
must be preserved. The incident records one deferred record-domain metadata test
failure; it is outside callable identity acceptance.
