class Canonical {
  construct new(value) { _value = value }
  value => _value
}
System.print(Canonical.new(6).value)
