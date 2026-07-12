// area: compile-errors
// spec: method-lookup.md §1.14; U-INH §3.4
// status: NEGATIVE
// U-INH: a bare `super` is not a value; it only redirects a send's lookup start.
class X {
  construct new() { }
  m => super
}
System.print(X.new().m)
