// area: classes
// spec: classes.md; selectors.md
// status: PENDING

@construct
class Point {
  mut _x
  mut _y
  @get mut _label
}

let p = Point.new(x: 3, y: 4, label: "origin")
System.print(p.label)
