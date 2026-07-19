// area: classes
// spec: classes.md; selectors.md
// status: PASS

class Person {
  name => _name
  construct new(n) {
    _name = n
  }
  name=(v) {
    _name = v
  }
}
const p = Person.new("Ada")
p.name = "Bob"
System.print(p.name)
