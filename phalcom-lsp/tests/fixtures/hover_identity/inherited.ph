class Animal {
  /// Animal speech documentation.
  speak() { }
}

class Dog is Animal {
  @constructor new() { }
}

let dog = Dog.new()
dog./*@speak*/speak()
