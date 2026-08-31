// area: messages/selectors
// spec: selectors.md; messages-and-selectors.md
// status: PASS

class Point {
  move(_ to, _ duration) {
    return "positional";
  }
  move(to, duration) {
    return "labeled";
  }
}
const p = Point.new()
System.print(p.move(1, 2))
System.print(p.move(to: 1, duration: 2))
