// Test repeated cached method resolution
class C {
  x => 42
}

let c1 = C.new()
let c2 = C.new()
let c3 = C.new()
let c4 = C.new()
let c5 = C.new()

System.print(c1.x)
System.print(c2.x)
System.print(c3.x)
System.print(c4.x)
System.print(c5.x)
