// area: errors
// spec: U9-implementation-spec.md §3, §6; messages-and-selectors.md §4
// status: NEGATIVE
// A rest parameter (`*rest`) must be the list's last entry — a clean parser
// diagnostic, not a panic.

class Bad {
  foo(*rest, x) {
    return x
  }
}
