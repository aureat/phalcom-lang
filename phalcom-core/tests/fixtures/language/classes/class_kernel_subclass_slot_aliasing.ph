class Sub is List {
  mut _tag
  @constructor
  new(_ t) { _tag = t }
  tag { _tag }
}

let mut s = Sub.new("ok")
System.print(s.tag)
