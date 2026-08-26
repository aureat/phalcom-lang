class Parent {
  run() {}
  parentOnly() {}
}

class Child is Parent {
  @constructor new() {}
  run() {}
  childOnly() {}
}

const child = Child.new()
child./*@completion*/run()
