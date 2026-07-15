// area: classes
// spec: classes.md §1; object-model.md §5; ADR-0002
// status: PASS
// A constructor resolves through *any* receiver expression, not only a
// literal class name.
//
// A constructor installs on the metaclass under the ordinary selector its
// call sites encode, so the parallel metaclass tower (ADR-0002) resolves
// `C.new()` to it — shadowing the bare allocator `Class >> new()` at the
// tower root — regardless of whether the receiver is spelled as the class's
// own name, a variable holding the class, or a list element. Regression: only
// a bare-identifier receiver used to reach the constructor; every other shape
// silently bare-allocated an instance with unset fields.

class Counter {
  construct new(start) {
    _n = start
  }
  n => _n
}

// Literal class-name receiver.
System.print(Counter.new(1).n)

// Variable receiver.
var C = Counter
System.print(C.new(2).n)

// Collection-element receiver.
var classes = [Counter]
System.print(classes[0].new(3).n)
