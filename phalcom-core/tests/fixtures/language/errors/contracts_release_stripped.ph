// flags: --release
// area: errors
// spec: contract-annotations.md
// status: PASS
// contract: under --release, @requires stays woven but @ensures/@invariant
// are stripped (U-ANNOT-CONTRACTS plan §3.6) — a postcondition or invariant
// that would fail in Debug mode now runs without raising, while an invalid
// argument still raises PreconditionError.

class Box {
  @invariant(self.val >= 0)

  @constructor
  new(_ init) {
    _val = init
  }

  val { _val }

  @requires(amount > 0)
  @ensures(false)
  badSet(_ amount) {
    _val = amount
  }

  breakInvariant() {
    _val = -1
  }
}

const b = Box.new(5)
System.print(b.val)

// @ensures(false) would always raise PostconditionError in Debug mode —
// stripped under --release, so this succeeds instead.
b.badSet(10)
System.print(b.val)

// Directly violates the class @invariant — stripped under --release, so
// this succeeds instead of raising InvariantError.
b.breakInvariant()
System.print(b.val)

// @requires stays woven in --release: an invalid argument still raises.
try {
  b.badSet(-1)
} catch e {
  System.print("PreconditionError: " + e.message)
}
