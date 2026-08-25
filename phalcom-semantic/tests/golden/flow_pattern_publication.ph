// LAW CHAIN
// 1. Factory branches join precise tuple components.
// 2. Tuple decomposition creates independent PatternDecomposition bindings.
// 3. Service return edge contributes no continuing value.
// 4. Factory -> Service -> Probe publishes one composed result.

class Animal {}
class Cat is Animal { @constructor new() {} }
class Dog is Animal { @constructor new() {} }

class Factory {
  @class
  pair(_ flag: Bool) {
    if flag {
      (Cat.new(), 1)
    } else {
      (Dog.new(), 2)
    }
  }
}

class Service {
  @class
  choose(_ flag: Bool, _ abort: Bool) {
    let pair = Factory.pair(flag)
    let (animal, count) = pair

    let result = if abort {
      return (animal, count)
    } else {
      (animal, count)
    }

    result
  }
}

class Probe {
  @class
  run(_ flag: Bool, _ abort: Bool) {
    let result = Service.choose(flag, abort)
    let (animal, count) = result
    (animal, count)
  }
}
