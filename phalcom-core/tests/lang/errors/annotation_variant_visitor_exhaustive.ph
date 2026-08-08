// area: errors
// spec: annotations-data.md §"@variant" / §"Visitor dispatch"
// status: PASS
// U-ANNOT-LAYOUT step 7: `@variant Name(labels...)` inside a `@sealed` class
// body expands into a sibling top-level class (itself `@data`), and the
// enclosing class gains a generated `match(...)` visitor that
// double-dispatches to each variant's own `__matchArm` override.

@sealed
@data
class Shape {
  @variant Circle(radius:)
  @variant Rect(w:, h:)
}

const c = Circle.new(radius: 3)
const r = Rect.new(w: 4, h: 5)

System.print(c.match(circle: |circ| { 3 * circ.radius }, rect: |rec| { rec.w * rec.h }))
System.print(r.match(circle: |circ| { 3 * circ.radius }, rect: |rec| { rec.w * rec.h }))
System.print(c.toString)
System.print(r.toString)
