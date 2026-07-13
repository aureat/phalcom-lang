// area: classes
// spec: next/is-tests.md; object-model.md §8
// status: PASS
// U-IS: the metaclass-tower discriminator. A class is itself an object; its
// direct class is its own metaclass (not `Class` itself), but `Class` sits
// on the metaclass's superclass chain — so a class is a kind-of `Class` but
// not exactly `Class`.

class Point {}

System.print(Point is Class)
System.print(Point is! Class)
