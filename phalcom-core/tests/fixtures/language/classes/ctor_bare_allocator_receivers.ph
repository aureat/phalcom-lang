class Empty {
  mut _value
  value { _value }
}
let mut C = Empty
let mut xs = [Empty]
System.print(C.new().value)
System.print(xs[0].new().value)
