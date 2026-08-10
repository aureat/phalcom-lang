// area: errors
// spec: contract-annotations.md, ADR-0052 unwind-safety regression
// status: PASS
// contract: a thrown error inside an @invariant-checked call must not leave the guard permanently inflated

class Vault {
  @invariant(self.balance >= 0)

  @constructor
  new(_ init) {
    _balance = init
  }

  balance { _balance }

  withdraw(_ amount) {
    if (amount > _balance) {
      throw Error.new("insufficient funds")
    }
    _balance = _balance - amount
  }
}

const v = Vault.new(10)

try {
  v.withdraw(999)
} catch e {
  System.print("caught: " + e.message)
}

// Guard must not be stuck "checking" after the unwind — a normal call afterwards
// must still run its invariant check.
v.withdraw(4)
System.print(v.balance)
