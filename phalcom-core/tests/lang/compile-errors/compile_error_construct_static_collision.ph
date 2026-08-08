// area: compile-errors
// spec: classes.md §1; object-model.md §5; ADR-0002
// status: NEGATIVE
// A `construct` and a `static` method of the same name/arity encode to one
// class-side selector and both install on the metaclass, so the later would
// silently clobber the earlier. The pair is a compile error rather than a
// declaration-order coin flip.

class Foo {
  @constructor
  new() {
    _x = 1
  }
  @class
  new() {
    return "shadowed"
  }
  x => _x
}
System.print(Foo.new().x)
