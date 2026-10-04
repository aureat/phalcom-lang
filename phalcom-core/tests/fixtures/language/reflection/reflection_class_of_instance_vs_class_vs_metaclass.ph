// area: reflection
// spec: object-model.md §5 (metaclass tower)
// status: PASS
// `class`/`class.name` walked one rung at a time: an instance's `.class`
// is its defining class; a class's `.class` is its metaclass (named
// `<ClassName> class`); a metaclass's own `.class` is `Class` itself.

class Point {}
let p = Point.new()
System.print(p.class.name)
System.print(Point.class.name)
System.print(Point.class.class.name)
