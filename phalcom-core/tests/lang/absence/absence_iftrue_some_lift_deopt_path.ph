// area: absence
// spec: values-and-absence.md §3; catalog-delta.md §4.2; ADR-0018 (sacred
//   selector inliner) amendment; invariant-requirements.md R-INV-2.1
// status: PASS
// U-CORE-2: R-INV-2.1 (deopt-path half). Reopening `Bool` to (re)install a
// sacred selector other than `ifTrue`/`ifFalse` flips `bool_sacred_pristine`
// (`universe.rs::note_method_installed`, called from the reopen's method
// install), so every inlined `Bool` guard site (`vm.rs` `GuardBool`) deopts
// to a real send. That send still resolves to the kernel `bool_if_true`/
// `bool_if_false` primitives (the reopen only redefines `and`, a *different*
// sacred selector), so the Some-lift is now exercised through the primitive
// path instead of `WrapSome`. Same `.expected` as the fast-path twin
// (`absence_iftrue_some_lift_fast_path.ph`) — that identity IS the
// assertion: fast path ≡ deopt path.

class Bool { and(other) { return false } }
System.print(true.ifTrue { 42 }.isSome)
System.print(false.ifTrue { 42 }.isSome)
System.print(false.ifTrue { 42 }.isNone)
System.print(true.ifTrue { }.isSome)
