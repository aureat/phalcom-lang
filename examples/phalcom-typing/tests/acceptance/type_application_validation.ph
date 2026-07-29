import "../../src/typing" as Typing

@protocol
class Vehicle {
  drive() -> None
}

class Car {
  drive() -> None {}
}

class Garage<T: Vehicle> {}
class DatabaseId<T in (Int, String)> {}

assert(Garage<Car>.origin === Garage)
assert(DatabaseId<Int>.arguments == const [Int])
assert(DatabaseId<String>.arguments == const [String])

assertThrows(Typing.TypeBoundError) {
  Garage<String>
}

assertThrows(Typing.TypeConstraintError) {
  DatabaseId<Float>
}
