class FieldBase {
  mut _count: Int
  @constructor new() { _count = 5 }
  value() { _count }
}

class FieldChild is FieldBase {
  _label: String = "child"
  @constructor new() { super.new() }
  label() { _label }
}

let child = FieldChild.new()
System.print(child.value())
System.print(child.label())
