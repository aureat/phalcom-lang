class A {
  val => 1
}

const a = A.new()

// Cache multiple times
const _ = a.val
const _ = a.val
const _ = a.val

// Reopen and change the method
class A {
  val => 2
}

// Should get the new value
System.print(a.val)  // Expect 2
