// area: classes
// spec: classes.md; selectors.md
// status: PENDING

@construct
class Point {
  var x
  var y
  @get var label
}

let p = Point.new(x: 3, y: 4, label: "origin")
System.print(p.label)
