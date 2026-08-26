// LAW CHAIN
// 1. AnimalMaker<T> inherits Maker<T>.
// 2. Nested calls specialize T to Cat and publish Box<Cat>.
// 3. Covariance validates Box<Cat> against Box<Animal>.
// 4. SelfNode#boxed specializes Self to CatNode.

class Animal {}

class Cat is Animal { @constructor new() {} }

class Box<+T> {
  _value: T
  @constructor new(_ value: T) { _value = value }
  value() -> T { _value }
}

class Maker<T> {
  wrap(_ value: T) -> Box<T> {
    Box<T>.new(value)
  }
}

class AnimalMaker<T> is Maker<T> {
  echo(_ value: T) -> T { value }
}

class SelfNode {
  @constructor new() {}
  boxed() -> Box<Self> {
    Box<Self>.new(self)
  }
}

class CatNode is SelfNode {}

class Service {
  @class
  makeCat(_ maker: AnimalMaker<Cat>) -> Box<Animal> {
    maker.wrap(maker.echo(Cat.new()))
  }

  @class
  makeNode() {
    CatNode.new().boxed()
  }
}

class Probe {
  @class
  run(_ maker: AnimalMaker<Cat>) {
    let animals: Box<Animal> = Service.makeCat(maker)
    let animal = animals.value()

    let nodeBox = Service.makeNode()
    let node = nodeBox.value()

    (animal, node)
  }
}
