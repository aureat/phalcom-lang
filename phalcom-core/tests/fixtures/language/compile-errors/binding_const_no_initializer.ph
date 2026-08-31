// area: compile-errors
// spec: ADR-0064 §binding grammar; U-BINDINGS §2.3
// status: NEGATIVE
// `const` requires an initializer at declaration — there is no uninitialized
// immutable binding.

const x
System.print(x)
