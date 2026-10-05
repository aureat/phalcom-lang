// LAW CHAIN
// 1. BoxOf is a type lambda with binder T and body Box<T>.
// 2. BoxOf<Cat> beta-reduces to Box<Cat>.
// 3. Covariance validates Box<Cat> against Box<Animal>.
// 4. Constrained<Cat> satisfies T <: Animal and publishes Cat members.

class Animal {}
class Cat is Animal { @constructor new() {} }

class Box<+T> {
  mut _value: T
  @constructor new(_ value: T) { _value = value }
  value() -> T { _value }
}

type BoxOf = <T> =>> Box<T>

class Constrained<T> where T <: Animal {
  mut _value: T
  @constructor new(_ value: T) { _value = value }
  value() -> T { _value }
}

class Factory {
  @class
  cat() -> BoxOf<Cat> {
    Box<Cat>.new(Cat.new())
  }

  @class
  constrained() -> Constrained<Cat> {
    Constrained<Cat>.new(Cat.new())
  }
}

class Probe {
  @class
  run() {
    let mut box = Factory.cat()
    let mut broad: Box<Animal> = box
    let mut cat = box.value()

    let mut constrained = Factory.constrained()
    let mut constrainedCat = constrained.value()

    (cat, constrainedCat)
  }
}
