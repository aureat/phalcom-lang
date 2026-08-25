// LAW CHAIN
// 1. Producer branches join Producer<Cat> and Producer<Dog>.
// 2. Covariance validates the broad Producer<Animal> contract.
// 3. Producer<String> is refuted without erasing actual producer knowledge.
// 4. Independent literal and downstream publication remain recoverable.

class Animal {}
class Cat is Animal { @constructor new() {} }
class Dog is Animal { @constructor new() {} }

class Producer<+T> {
  _value: T
  @constructor new(_ value: T) { _value = value }
  value() -> T { _value }
}

class Shelter {
  @class
  choose(_ flag: Bool) {
    if flag {
      Producer<Cat>.new(Cat.new())
    } else {
      Producer<Dog>.new(Dog.new())
    }
  }
}

class Service {
  @class
  run(_ flag: Bool) {
    let producer: Producer<Animal> = Shelter.choose(flag)
    let animal = producer.value()

    let bad: Producer<String> = Shelter.choose(flag)
    let independent = 42

    (animal, independent)
  }
}

class Probe {
  @class
  run(_ flag: Bool) {
    Service.run(flag)
  }
}
