class Animal {
  move() {}
}

class Dog is Animal {
  bark() {}
}

const dog = Dog.new()
dog.bark()
