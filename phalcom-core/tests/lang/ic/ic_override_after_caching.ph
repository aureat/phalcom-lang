class A {
  get => 1
}

const a = A.new()

// Cache by repeated calls
const _ = a.get
const _ = a.get
const _ = a.get
const _ = a.get
const _ = a.get
const _ = a.get
const _ = a.get
const _ = a.get
const _ = a.get
const _ = a.get

// Reopen and override
class A {
  get => 2
}

System.print(a.get)  // Expect 2
