class Empty {
  _value
  value => _value
}
let C = Empty
let xs = [Empty]
System.print(C.new().value)
System.print(xs[0].new().value)
