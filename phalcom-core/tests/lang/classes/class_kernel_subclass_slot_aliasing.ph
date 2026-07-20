class Sub extends List {
  _tag
  construct new(t) { _tag = t }
  tag => _tag
}

let s = Sub.new("ok")
System.print(s.tag)
