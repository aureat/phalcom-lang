class Canonical {
  @constructor
  new(value) { _value = value }
  value => _value
}
System.print(Canonical.new(6).value)
