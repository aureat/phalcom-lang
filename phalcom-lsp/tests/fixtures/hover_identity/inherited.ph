class Animal {
  /// Animal speech documentation.
  speak() { }
}

class Dog is Animal {
  @constructor new() { }
}

const dog = Dog.new()
dog./*@speak*/speak()
