// area: errors
// spec: annotations-construct.md
// status: PASS
// U-ANNOT-LAYOUT step 3: @construct derives a real constructor from declared
// fields, own-fields-only (no superclass chaining).

@construct
class Point {
  var _x
  var _y

  sum => _x + _y
}

System.print(Point.new(x: 3, y: 4).sum)
