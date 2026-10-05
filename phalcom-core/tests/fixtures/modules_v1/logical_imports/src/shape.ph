class Shape {
  mut _sides
  @constructor
  new(_ sides) {
    _sides = sides
  }
  sides { _sides }
}
export Shape
