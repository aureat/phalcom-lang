// area: compile-errors
// spec: objects/enums.md
// status: NEGATIVE
// Sum cases belong in `data` declarations; class bodies cannot declare them.

class Shape {
  @variant Circle(radius:)
}
