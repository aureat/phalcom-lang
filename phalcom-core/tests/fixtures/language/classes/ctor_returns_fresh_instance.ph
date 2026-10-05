class Box {
  mut _value
  @constructor
  new(_ value) {
    _value = value
    "body value"
  }
  value { _value }
}
System.print(Box.new(9).value)
