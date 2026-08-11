import "./module" as Module

class Base {
  base() {}
}

class Widget is Base {
  @constructor
  new(_ value) {
    _value = value
  }

  @class
  make() {
    Widget.new(42)
  }

  value { _value }

  value=(put next) {
    _value = next
  }

  move(_ x, to dest) {
    const tuple = (x, dest)
    const list = [1, 2, 3]
    const record = #{ name: "widget", value: _value }
    const selector = #move(_,to)
    const block = || { "done" }
    list[0]
    tuple
    record
    selector
    block
  }

  [_ index] {
    return index
  }

  [_ index]=(put value) {
    _value = value
  }
}

const instance = Widget.new(1)
instance.move(2, to: 3)
