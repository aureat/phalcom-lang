// area: classes
// spec: object-model.md; method-lookup.md
// status: PENDING

class Animal {
  speak() {
    return "...";
  }
}

class Dog : Animal {
  speak() {
    return super.speak() + " Woof";
  }
}

System.print(Dog.new().speak())
