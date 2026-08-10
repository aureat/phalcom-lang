class Foo {
  @constructor
  new() { _value = 0 }
  @constructor
  new(_ value) { _value = value }
  value { _value }
}
System.print(Foo.new().value)
System.print(Foo.new(9).value)
