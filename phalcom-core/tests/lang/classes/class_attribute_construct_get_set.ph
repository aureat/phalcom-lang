// area: classes
// spec: classes.md; selectors.md
// status: PENDING

@construct
class Point {
  _x
  _y
  @get _label
}

const p = Point.new(x: 3, y: 4, label: "origin")
System.print(p.label)
