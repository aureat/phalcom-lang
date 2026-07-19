// area: compile-errors
// spec: classes.md §1; ADR-0011
// status: NEGATIVE
// U7: a class that declares a `new`-named `construct` has no user-visible
// bare allocator — calling `new()` with arity/labels that match none of its
// declared constructors must not silently fall through to the inherited
// `Object::new` bare-allocation primitive; it is a compile error.

class Widget {
  construct new(label:) { _label = label }
  label => _label
}
const w = Widget.new()
System.print(w)
