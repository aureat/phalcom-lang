// area: errors
// spec: F.3-rest-capture-and-rest-pattern-dispatch-amended.md §9
// status: NEGATIVE
// Mixing a labeled (keyword) parameter with a following rest parameter is
// rejected because positional rest must precede labeled parameters. A clean
// parser diagnostic, not a panic.

class Bad {
  foo(to, *rest) {
    return rest
  }
}
