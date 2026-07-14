// area: runtime-errors
// spec: error-handling.md §1, U-STRING
// status: NEGATIVE
// Uncaught ArgumentError renders its message and exits non-zero.

throw ArgumentError.new("invalid argument value")
