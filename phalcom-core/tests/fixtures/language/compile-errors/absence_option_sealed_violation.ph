// area: compile-errors
// spec: values-and-absence.md §3.1; U-ANNOT-LAYOUT §3.4 (`attr.sealed_violation`)
// status: NEGATIVE
// `Option` is sealed to the core module at bootstrap — user code must not
// extend it.

class MyOpt is Option<Int> {}
