class A {
  get => 1
}

let a = A.new()

// Cache by repeated calls
let _ = a.get
let _ = a.get
let _ = a.get
let _ = a.get
let _ = a.get
let _ = a.get
let _ = a.get
let _ = a.get
let _ = a.get
let _ = a.get

// Reopen and override
class A {
  get => 2
}

System.print(a.get)  // Expect 2
