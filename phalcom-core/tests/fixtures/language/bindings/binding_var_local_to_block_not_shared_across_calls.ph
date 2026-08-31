// area: bindings
// spec: values-and-absence.md; blocks.md §5; ADR-0014
// status: PASS
// A `let` DECLARED INSIDE a block's own body (not captured from an
// enclosing scope) is a fresh local slot on every call — the opposite of
// `blocks_shared_upvalue_two_closures.ph` (an OUTER `let` shared across
// closures). Each `.call()` re-enters with a brand-new `localVar`, so
// mutating it inside the block never persists to the next call.

const bump = || {
  let localVar = 5
  localVar = localVar + 1
  System.print(localVar)
}
bump.call()
bump.call()
bump.call()
