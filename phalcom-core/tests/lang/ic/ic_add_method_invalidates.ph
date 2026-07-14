class A {
  val => 1
}

let a = A.new()

// Cache multiple times
let _ = a.val
let _ = a.val
let _ = a.val

// Reopen and change the method
class A {
  val => 2
}

// Should get the new value
System.print(a.val)  // Expect 2
