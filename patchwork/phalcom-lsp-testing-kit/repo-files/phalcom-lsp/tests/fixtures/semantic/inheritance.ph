class Animal {
  move() {}
  name { "animal" }
}

class Dog is Animal {
  bark() {}
}

const dog = Dog.new()
dog./*@completion*/bark()
