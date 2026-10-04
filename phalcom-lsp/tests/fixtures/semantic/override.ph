class Parent {
  run() {}
  parentOnly() {}
}

class Child is Parent {
  @constructor new() {}
  run() {}
  childOnly() {}
}

let child = Child.new()
child./*@completion*/run()
