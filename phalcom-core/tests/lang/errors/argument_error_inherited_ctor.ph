// area: errors
// spec: error-handling.md §1, U-STRING
// status: PASS
// ArgumentError — the boundary-guard exception class (U-STRING).
// Inherits Error's construct new(msg) via U-INH inherited-ctor resolution.

let err = ArgumentError.new("age must be >= 0")
System.print(err.message)
System.print(err.isA(Error))
System.print(err.isA(ArgumentError))
