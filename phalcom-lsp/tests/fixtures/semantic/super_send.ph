class Grandparent {
  grandOnly() {}
}

class Parent is Grandparent {
  parentOnly() {}
  shared() {}
}

class Child is Parent {
  childOnly() {}

  shared() {
    super./*@super*/shared()
  }
}
