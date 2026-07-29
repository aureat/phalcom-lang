// Executable design acceptance for the future generic-aware Phalcom toolchain.
// The Python verifier checks this file structurally until that toolchain exists.

import Type, TypeParameter, AppliedType from typing

class Box<T> {
  const _value: T

  @constructor
  new(value: T) {
    _value = value
  }

  value -> T {
    return _value
  }

  @class
  applicationSeen() -> Option<AppliedType> {
    return Type.currentApplication
  }
}

const parameter: TypeParameter = Box.typeParameters.first
assert(parameter.name == #T)
assert(parameter.owner === Box)
assert(parameter.index == 0)
assert(parameter.variance == Variance.Invariant)

const intBox = Box<Int>
assert(intBox.origin === Box)
assert(intBox.arguments == const [Int])
assert(intBox === Box<Int>)
assert(intBox.methodFor(#value).returnType == Int)

const box = Box<Int>.new(value: 42)
assert(box.class === Box)
assert(box.value == 42)
assert(Box<Int>.applicationSeen == Some.new(Box<Int>))
assert(Box.applicationSeen == None)
