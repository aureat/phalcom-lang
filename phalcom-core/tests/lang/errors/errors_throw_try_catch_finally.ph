// area: errors
// spec: error-handling.md §1-§4; ADR-0031
// status: PASS
// Graduated from `pending/` (U-CORE-6 §4 reserved this name): rewritten onto
// the *ratified* ADR-0031 surface (`try`/`on T e {}`/`catch e {}`/`ensure{}`)
// rather than the placeholder `catch (e: T) {} finally {}` JS-style spelling
// the pending draft carried before ADR-0031 fixed the actual grammar.
class ArgumentError extends Error {
  construct new(msg) { super.new(msg) }
}

try {
  throw ArgumentError.new("age must be >= 0")
} on ArgumentError e {
  System.print(e.message)
} ensure {
  System.print("done")
}
