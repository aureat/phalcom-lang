class Foo {
  mut _value
  @constructor
  new(_ value) { _value = value }
  @class
  new(_ value) { 1 }
}
