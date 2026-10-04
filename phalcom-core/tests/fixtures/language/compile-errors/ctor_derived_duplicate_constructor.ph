@construct
class Foo {
  mut _value
  @constructor
  new(value) { _value = value }
}
