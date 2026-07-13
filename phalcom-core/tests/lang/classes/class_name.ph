// area: classes
// spec: object-model.md §5 (metaclass tower); classes.md
// status: PASS
// Ported from Wren `test/core/class/name.wren`: a class's own `name`, plus
// the built-in classes' names, and their metaclasses' names. Pinned as
// observed: a user-defined class's metaclass renders its name with a `.`
// (`Foo.class`, `vm.rs`'s `name + ".class"`), while a bootstrap class's
// metaclass renders with a space (`Object class`) — the two naming schemes
// are not unified in the current implementation.

class Foo {}

System.print(Foo.name)
System.print(Foo.class.name)

// Make sure the built-in classes have proper names too.
System.print(Object.name)
System.print(Bool.name)
System.print(Class.name)

// And metaclass names.
System.print(Object.class.name)
System.print(Bool.class.name)
