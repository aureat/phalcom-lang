class Animal {}
class Dog is Animal {}
let d = Dog.new()
System.print(d is in (Animal, Dog))
System.print(d is! in (Dog, String))
System.print(d is! in (Animal, String))
