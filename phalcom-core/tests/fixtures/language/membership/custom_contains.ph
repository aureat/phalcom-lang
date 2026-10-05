class CustomBox {
  mut _val
  @constructor
  new(_ val) { _val = val }
  contains(_ x) { _val == x }
}
let mut box = CustomBox.new(42)
System.print(42 in box)
System.print(99 in box)
System.print(99 not in box)
