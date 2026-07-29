class Sub is List {
  _tag
  @constructor
  new(t) { _tag = t }
  tag => _tag
}

let s = Sub.new("ok")
System.print(s.tag)
