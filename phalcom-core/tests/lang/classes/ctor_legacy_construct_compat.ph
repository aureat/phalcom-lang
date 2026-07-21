class Legacy {
  construct new(value) { _value = value }
  value => _value
}
System.print(Legacy.new(6).value)
