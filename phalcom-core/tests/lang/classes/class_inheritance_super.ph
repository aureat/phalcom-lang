// area: classes
// spec: object-model.md; method-lookup.md
// status: PASS

class Animal {
  speak() {
    return "...";
  }
}

class Dog is Animal {
  speak() {
    return super.speak() + " Woof";
  }
}

System.print(Dog.new().speak())
