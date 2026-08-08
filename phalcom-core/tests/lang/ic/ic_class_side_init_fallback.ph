// Test repeated cached method resolution
class C { x => 42
}

const c1 = C.new()
const c2 = C.new()
const c3 = C.new()
const c4 = C.new()
const c5 = C.new()

System.print(c1.x)
System.print(c2.x)
System.print(c3.x)
System.print(c4.x)
System.print(c5.x)
