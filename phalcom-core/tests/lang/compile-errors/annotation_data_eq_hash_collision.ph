// area: compile-errors
// spec: annotations-data.md §"@data" (Hazards: "==/hash derived together only")
// status: NEGATIVE
// `==`/`hash` are derived together or not at all — a class hand-writing one
// and deriving the other via `@data` is `attr.accessor_collision`.

@data
class Money {
  _cents

  ==(other) {
    return _cents == other.cents
  }
}
