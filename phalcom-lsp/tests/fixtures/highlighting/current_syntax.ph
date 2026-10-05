import .module as Module

class Base {
  base() {}
}

class Widget is Base {
  mut _value
  @constructor
  new(_ value) {
    _value = value
  }

  @class
  make() {
    Widget.new(42)
  }

  value { _value }

  value=(_ next) {
    _value = next
  }

  move(_ x, to dest) {
    let tuple = (x, dest)
    let list = [1, 2, 3]
    let record = { name: "widget", value: _value }
    let selector = #move(_,to)
    let block = || { "done" }
    list[0]
    tuple
    record
    selector
    block
  }

  [_ index] {
    return index
  }

  [_ index]=(_ value) {
    _value = value
  }
}

let instance = Widget.new(1)
instance.move(2, to: 3)
