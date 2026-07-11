// area: classes
// spec: classes.md; selectors.md
// status: PENDING

@construct
class Point {
  var x
  var y
  @get var label
}

let p = Point.new(3, 4, "origin")
System.print(p.label)
