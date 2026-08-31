// area: errors
// spec: annotations-data.md @variant / this implementation specification
// status: PASS

enum Shape {
  @variant Circle(radius: Int)
  @variant Rect(w: Int, h: Int)
}

const c = Shape::Circle(radius: 3)
const r = Shape::Rect(w: 4, h: 5)

System.print(3 * c.radius)
System.print(r.w * r.h)
System.print(c.toString)
System.print(r.toString)
