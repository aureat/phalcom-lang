// area: metaclass
// spec: object-model.md
// status: PASS

class Point {
}
System.print(Point.class.superclass == Point.superclass.class)
