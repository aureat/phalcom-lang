// area: inheritance
// spec: object-model.md §5 rule 4; method-lookup.md §1.14; ADR-0002 (parallel metaclass)
// status: PASS
// U-ERR-FIX SUPER-STATIC: `super.<name>` from inside a `static` override
// resolves up the class-side (metaclass) tower, not the instance-side one.
// A three-level static override chain, each calling `super`, proves the
// walk starts at the *metaclass's* superclass.

class Animal {
  @class
  greet => "hi"
}
class Dog is Animal {
  @class
  greet => super.greet + "-dog"
}
class Puppy is Dog {
  @class
  greet => super.greet + "-puppy"
}
System.print(Dog.greet)
System.print(Puppy.greet)
