// area: compile-errors
// spec: classes.md §1; ADR-0011; U-INH §3.5
// status: NEGATIVE
// U-INH follow-on: the "no user-visible bare allocator" rule is
// inheritance-aware. A subclass that INHERITS a `new`-named `construct` but
// declares none of its own still has no bare allocator — a `new(...)` call
// whose arity/labels match no ancestor `construct` must be a compile error,
// not a silent fall-through to `Object.class::new` (which would return an
// uninitialized instance). Here `Sub` inherits `new(label:)` from `Base` and
// declares no constructor; `Sub.new()` matches no ancestor `construct`.

class Base {
  construct new(label:) { _label = label }
  label => _label
}
class Sub extends Base {
}
const s = Sub.new()
System.print(s)
