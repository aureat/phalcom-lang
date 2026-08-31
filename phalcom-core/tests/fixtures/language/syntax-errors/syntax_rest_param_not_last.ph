// area: errors
// spec: F.3-rest-capture-and-rest-pattern-dispatch-amended.md §9
// status: NEGATIVE
// A rest parameter (`*rest`) must be the list's last entry — a clean parser
// diagnostic, not a panic.

class Bad {
  foo(*rest, _ x) {
    return x
  }
}
