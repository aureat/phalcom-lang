class Parent {
  run() {}
  parentOnly() {}
}

class Child is Parent {
  run() {}
  childOnly() {}
}

const child = Child.new()
child./*@completion*/run()
