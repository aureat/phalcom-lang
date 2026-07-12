// area: compile-errors
// spec: error-handling.md §1; ADR-0031 §1
// status: NEGATIVE
// `throw` of a syntactically-detectable non-`Error` literal is a compile
// error — only `Error` subclasses are throwable.
throw "oops"
