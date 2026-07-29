import "../../src/typing" as Typing

class Product {
  const _value

  @constructor
  new(value) {
    _value = value
  }
}

class Factory<T> {
  @class
  make(value: T) -> Product {
    assert(Typing.Type.currentApplication == Some.new(Factory<T>))
    return Product.new(value: value)
  }

  @class
  makeIndirect(value: T) -> Product {
    return self.make(value: value)
  }
}

const value = Factory<Int>.make(value: 42)
assert(value.class === Product)
assert(Factory<Int>.makeIndirect(value: 42).class === Product)
assert(Typing.Type.currentApplication == None)
