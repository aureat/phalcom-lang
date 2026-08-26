class Animal {
  move() {}
  name { "animal" }
}

class Dog is Animal {
  @constructor new() {}
  bark() {}
}

const dog = Dog.new()
dog./*@completion*/bark()
