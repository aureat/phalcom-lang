// area: compile-errors
// spec: classes.md §1-2; ADR-0011
// status: NEGATIVE
// U7: whole-class field collection only ever assigns `_naem` (a typo) —
// no member of `Typo` ever writes `_name`, so reading it is a compile error
// (catches the private-field typo instead of silently reading `None`).

class Typo {
  @constructor
  new(_ name) { _naem = name }
  name => _name
}
System.print(Typo.new("x").name)
