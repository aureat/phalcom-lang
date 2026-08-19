// area: absence
// spec: values-and-absence.md §3; object-model.md; ADR-0007
// status: PASS
// Adversarial: `None` (the surface absence singleton) is an ordinary Object
// citizen, distinct from the private `Value::Nil` sentinel that has no
// surface syntax (Invariant 4, cf. `compile-errors/compile_error_surface_nil`).
// `None` is `isA(Object)`, answers `respondsTo(_:)` like any instance, and
// its own class name is `"None"`.

System.print(None.is(Object))
System.print(None.respondsTo(Symbol.new("toString")))
System.print(None.class.name)
