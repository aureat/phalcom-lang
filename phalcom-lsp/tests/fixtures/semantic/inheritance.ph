class Animal {
  move() {}
  name { "animal" }
}

class Dog is Animal {
  @constructor new() {}
  bark() {}
}

let dog = Dog.new()
dog./*@completion*/bark()
