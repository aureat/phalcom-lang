// area: errors
// spec: U9-implementation-spec.md §3, §6; messages-and-selectors.md §4
// status: NEGATIVE
// Mixing a labeled (keyword) parameter with a following rest parameter is
// rejected — the variadic selector encoding ignores labels entirely (U9
// corrections §0 point 3), so this combination could never dispatch
// correctly. A clean parser diagnostic, not a panic.

class Bad {
  foo(to:, *rest) {
    return rest
  }
}
