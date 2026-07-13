// area: classes
// spec: object-model.md; method-lookup.md
// status: PASS
// Ported from Wren `test/core/class/supertype.wren`: a class with no
// explicit `extends` inherits `Object`; otherwise `superclass` is the
// declared parent; the root `Object` has no superclass — `None`, not the
// raw `nil` Wren's `null` would be (ADR-0007).

class Foo {}

class Bar extends Foo {}

class Baz extends Bar {}

// A class with no explicit superclass inherits Object.
System.print(Foo.superclass == Object)

// Otherwise, it's the superclass.
System.print(Bar.superclass == Foo)
System.print(Baz.superclass == Bar)

// Object has no supertype.
System.print(Object.superclass)
