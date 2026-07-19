// flags: --unchecked
// area: errors
// spec: contract-annotations.md
// status: PASS
// contract: under --unchecked, all three contract guards (@requires,
// @ensures, @invariant) are stripped (U-ANNOT-CONTRACTS plan §3.6) — every
// call that would violate a contract in Debug mode now runs without raising,
// including the invalid-argument case that --release alone still catches.
// (Reflectable contract metadata — MethodObject::contracts — is stripped by
// default under --unchecked too; that half of this file's name is verified
// at the Rust level in tests/contracts_metadata.rs, since no `.ph`-visible
// reflection surface exists yet to observe it from source.)

class Box {
  @invariant(self.val >= 0)

  construct new(init) {
    _val = init
  }

  val => _val

  @requires(amount > 0)
  @ensures(false)
  badSet(amount) {
    _val = amount
  }

  breakInvariant() {
    _val = -1
  }
}

const b = Box.new(5)
System.print(b.val)

// @requires(amount > 0) is stripped under --unchecked — an invalid argument
// no longer raises PreconditionError.
b.badSet(-1)
System.print(b.val)

// @ensures(false) is stripped under --unchecked — no PostconditionError.
b.badSet(10)
System.print(b.val)

// @invariant is stripped under --unchecked — no InvariantError.
b.breakInvariant()
System.print(b.val)

System.print("ran without exception")
