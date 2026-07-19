// area: compile-errors
// spec: ADR-0064 §4 (L-2); U-BINDINGS §4
// status: NEGATIVE
// `let _x` is not a field form — mutable fields take no keyword.

class Widget {
  let _hidden
}
