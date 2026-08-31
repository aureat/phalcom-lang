// area: classes
// spec: next/is-tests.md
// status: PASS
// U-IS: `is` is subclass-inclusive (walks the superclass chain); `is!` is
// not (live direct-class identity only). A `Dog extends Animal` instance is
// a kind-of `Animal` but not exactly an `Animal`, and is exactly a `Dog`.

class Animal {}
class Dog is Animal {}

let d = Dog.new()
System.print(d is Animal)
System.print(d is! Animal)
System.print(d is not Animal)
System.print(d is! Dog)
System.print(d is! not Dog)
