// area: inheritance
// spec: object-model.md §5.1; method-lookup.md §1-2; ADR-0002
// status: PASS
// U-INH: `extends` gives a subclass its superclass's instance methods; an
// override on the subclass wins over the inherited definition.

class Animal {
  @constructor
  new() { }
  legs { 4 }
  describe { "an animal" }
}
class Dog is Animal {
  @constructor
  new() { }
  describe { "a dog" }
}
const d = Dog.new()
System.print(d.legs)
System.print(d.describe)
