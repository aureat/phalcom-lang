// area: absence
// spec: values-and-absence.md §3; catalog-delta.md §4.2; ADR-0018 (sacred
//   selector inliner) amendment; invariant-requirements.md R-INV-2.1
// status: PASS
// U-CORE-2: R-INV-2.1 (fast-path half). On the pristine (inlined) path,
// `Bool#ifTrue(_)` Some-lifts its taken arm and yields the `None` singleton
// on its untaken arm — the sacred inliner's `WrapSome` emission keeps the
// guarded fast path observationally identical to the primitive deopt path
// (see the twin `absence_iftrue_some_lift_deopt_path.ph`). Also folds in the
// empty-body taken-arm case: `Bytecode::Nil` pushes the `None` singleton, so
// `true.ifTrue || { }` inlines to `Nil; WrapSome` -> `Some(None)`, a legal
// `Some` that never trips the Invariant-4 sentinel-wrap assert.

System.print(true.ifTrue || { 42 }.isSome)
System.print(false.ifTrue || { 42 }.isSome)
System.print(false.ifTrue || { 42 }.isNone)
System.print(true.ifTrue || { }.isSome)
