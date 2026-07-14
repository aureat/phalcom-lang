// area: absence
// spec: values-and-absence.md §3; ADR-0018 (sacred selector inliner)
//   amendment; invariant-requirements.md R-INV-2.1
// status: PASS
// Nested conditionals on the DEOPT path (perf-log F13). Every sacred call emits
// its arms twice — inlined, and again as block literals for the `GuardBool`
// fallback. The fallback copy is compiled with the inliner suppressed, since
// inlining there is what made code size 2^depth in nesting depth. This fixture
// is the behavior half of that change: the fallback copy must still compute the
// same answers with its inner conditionals compiled as ordinary sends.
//
// Reopening `Bool` to install `and` flips `bool_sacred_pristine`
// (`universe.rs::note_method_installed`), so every `GuardBool` site deopts to a
// real send into the kernel `bool_if_true`/`bool_if_false` primitives, which
// call the arms as blocks — and those blocks hold the nested conditionals.
// Same `.expected` as the fast-path twin
// (`absence_iftrue_nested_fast_path.ph`); that identity IS the assertion.

class Bool { and(other) { return false } }

var n = 3
// 4-deep nest, the shape `String.codePointAt` has: each level's `ifFalse` arm
// carries the next conditional, so only the innermost taken arm produces a
// value.
var r = (n < 1).ifTrue({ "one" }, ifFalse: {
  (n < 2).ifTrue({ "two" }, ifFalse: {
    (n < 3).ifTrue({ "three" }, ifFalse: {
      (n < 4).ifTrue({ "four" }, ifFalse: { "big" })
    })
  })
})
System.print(r)

// One-armed nesting: the untaken arms surface `None` through the primitive.
System.print((n < 4).ifTrue({ (n < 2).ifTrue({ "inner" }) }).isSome)
System.print((n < 2).ifTrue({ (n < 4).ifTrue({ "inner" }) }).isNone)
