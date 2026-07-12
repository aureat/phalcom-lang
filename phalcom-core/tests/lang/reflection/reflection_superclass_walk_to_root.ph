// area: reflection
// spec: object-model.md §5; ADR-0007; ADR-0010
// status: PASS
// Walking `superclass` from a user-defined leaf class up through the
// built-in hierarchy to the root: `Grandchild -> Child -> Object -> None`.
// The root class's `superclass` is the `None` singleton (U6 Invariant 4),
// terminating the walk with a surface absence value, not a raw sentinel.

class Child {}
class Grandchild extends Child {}
System.print(Grandchild.superclass.name)
System.print(Grandchild.superclass.superclass.name)
System.print(Grandchild.superclass.superclass.superclass)
