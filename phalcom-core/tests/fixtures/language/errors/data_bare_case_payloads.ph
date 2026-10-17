// area: errors
// spec: objects/enums.md
// status: PASS

data Shape {
  Circle(radius: Int)
  Rect(w: Int, h: Int)
}

let c = Shape::Circle(radius: 3)
let r = Shape::Rect(w: 4, h: 5)

System.print(3 * c.radius)
System.print(r.w * r.h)
System.print(c.toString)
System.print(r.toString)
