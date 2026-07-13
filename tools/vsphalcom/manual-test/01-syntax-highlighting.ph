// 01-syntax-highlighting.ph
//
// Open this file in the Extension Development Host and run "Developer:
// Inspect Editor Tokens and Scopes" on each marked token below. Every one
// should resolve to a distinct, non-generic scope (not plain
// source.phalcom). Dead keywords (const/is/in/and/or/not) should NOT be
// colored as keywords anywhere in this file.

/// Outer doc: documents the class below (this whole comment block is a
/// distinct comment.block.documentation scope, not a plain `//` comment).
class Account extends Object {
  //! Inner doc: documents the enclosing class from inside its body.

  // _balance / _owner below are field identifiers (leading underscore) —
  // distinct scope from a bare `foo`. Fields are declared implicitly by
  // assignment (ADR-0011), not by a standalone declaration statement.

  @requires(amount > 0)                 // @ attribute token
  @ensures(self._balance == old(self._balance) + amount)
  deposit(amount) {
    self._balance = self._balance + amount
    return self
  }

  construct new(owner) {
    _owner = owner
    _balance = 0
  }

  toString {
    // NOTE: `return "..."` directly with a string-interpolation literal
    // currently fails to parse (return + \(expr) interaction) — assigning
    // to a local first is a workaround, unrelated to this fixture's own
    // purpose (see the spawned follow-up task on this bug).
    var s = "Account(\(self._owner), balance=\(self._balance))"   // \(expr) interpolation
    return s
  }

  // Symbol literals
  describeSelector {
    var sel = #deposit           // name symbol
    var full = #deposit(_)       // selector symbol, contiguous with (
    var op = #==                 // operator selector
    return sel
  }

  // Method reference `::`
  boundDeposit {
    var ref = self::deposit
    var pinned = self::#deposit(_)
    return ref
  }

  // Option operators
  maybeOwner(opt) {
    var name = opt?.name
    var fallback = opt ?? "unknown"
    return name ?? fallback
  }

  // Index sugar
  firstTransaction(list) {
    var first = list[0]
    list[0] = "replaced"
    return first
  }

  // Error-handling keywords
  safeWithdraw(amount) {
    try {
      self.withdraw(amount)
    } on InsufficientFunds e {
      System.print(e.message)
    } catch e {
      System.print("unexpected error")
    } ensure {
      System.print("done")
    }
  }

  withdraw(amount) {
    (amount > self._balance).ifTrue { throw InsufficientFunds.new("not enough funds") }
    self._balance = self._balance - amount
  }
}

// Keyword coverage: class extends super self static try catch on ensure
// throw break continue match return while for var
class Savings extends Account {
  static rate { return 0.05 }

  applyInterest() {
    var i = 0
    while (i < 3) {
      i = i + 1
      (i == 2).ifTrue { continue }
      (i == 3).ifTrue { break }
    }
    return super.toString
  }
}
