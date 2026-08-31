class Animal {}
class Dog is Animal {}
let d = Dog.new()
System.print(d is! not in (Animal, String))
System.print(d is! not in (Dog, String))
