class A {
  ping() { }
}

class B {
  ping() { }
}

let mystery = missing
mystery./*@ping*/ping()
