// area: compile-errors
// spec: object-model.md §5.1; U-INH §3.2
// status: NEGATIVE
// U-INH: a class cannot name itself as its superclass — that would make method
// lookup non-terminating.
class A extends A {
  construct new() { }
}
System.print(1)
