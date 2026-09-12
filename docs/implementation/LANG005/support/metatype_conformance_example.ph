// LANG005 — class-object / metatype conformance example.
//
// This is an example, not a mandatory kernel trait.
// `class Person` is placeholder source spelling for the ClassObject<Person>
// conformance target; LANG005 may choose another spelling.

trait Parser {
  type Output

  parse(_ source: String) -> Output
}

data Person {
  name: String
  age: Int
}

impl Parser for class Person {
  type Output = Person

  parse(_ source: String) -> Person {
    Person {
      name: source,
      age: 0,
    }
  }
}

trait Printable {
  print -> String
}

impl Printable for Person {
  print -> String {
    "\(self.name) (\(self.age))"
  }
}

// Semantically:
//
//   Person instance        conforms Printable
//   class object `Person`  conforms Parser
//
// Runtime metaclass inheritance does not itself imply either conformance.
