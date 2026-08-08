// area: compile-errors
// spec: annotations-data.md §"@variant"; annotations-legality-grammar.md
// status: NEGATIVE
// `@variant` is only meaningful inside a `@sealed` class body — a class
// declaring `@variant` arms with no `@sealed` marker is a compile error, not
// a silently-open hierarchy.

@data
class Shape {
  @variant Circle(radius)
}
