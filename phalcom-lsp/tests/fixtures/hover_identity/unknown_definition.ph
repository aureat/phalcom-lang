class A {
  ping() { }
}

class B {
  ping() { }
}

const mystery = missing
mystery./*@ping*/ping()
