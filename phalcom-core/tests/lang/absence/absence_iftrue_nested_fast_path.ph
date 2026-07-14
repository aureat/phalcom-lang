// area: absence
// spec: values-and-absence.md §3; ADR-0018 (sacred selector inliner)
//   amendment; invariant-requirements.md R-INV-2.1
// status: PASS
// Nested conditionals on the INLINED FAST path — the twin of
// `absence_iftrue_nested_deopt_path.ph`. Identical source minus the `Bool`
// reopen, so every `GuardBool` site takes the inlined arm instead of deopting
// to a send. Sharing one `.expected` with the deopt twin is the assertion:
// fast path ≡ deopt path, including when the arms themselves nest.
//
// The pair guards perf-log F13's fix, which stops inlining inside the
// deopt-fallback copy of a sacred call's arms (that copy inlining its own
// nested conditionals is what made compile time 2^depth).

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

// One-armed nesting: the untaken arms surface `None`.
System.print((n < 4).ifTrue({ (n < 2).ifTrue({ "inner" }) }).isSome)
System.print((n < 2).ifTrue({ (n < 4).ifTrue({ "inner" }) }).isNone)
