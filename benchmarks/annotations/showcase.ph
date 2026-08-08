//! showcase — the full Phalcom annotation surface, documented with Phaldoc.
//!
//! ┌─ NOT RUNNABLE TODAY ────────────────────────────────────────────────────┐
//! │ This file is a **visual design showcase**, deliberately kept OUT of      │
//! │ benchmarks/math/ so `run.sh` never executes it. It will not lex on the   │
//! │ current tree: the code-level `@`-attributes below need `Token::At` and   │
//! │ the desugar pass, which are unbuilt (annotations-core.md). The Phaldoc   │
//! │ `///`/`//!` comments ARE inert today; the `@requires`/`@construct`/etc.  │
//! │ lines are not. Do not wire this into CI until the `@` lexer lands.       │
//! └─────────────────────────────────────────────────────────────────────────┘
//!
//! It shows the two annotation worlds side by side, per Phaldoc §8:
//!   • `///` / `//!`  → intent (prose) — inert comment convention
//!   • `@…`           → machine-checkable facts (contracts, layout, bridges)
//! and the rule that they must never restate each other.
//!
//! Specs: doc-comments-phaldoc.md, annotations-contracts.md,
//! annotations-construct.md, annotation-paradigm-bridges.md.

// ── Design by Contract (method-table-macro tier) ──────────────────────────────

/// A single-currency bank account whose balance can never go negative.
///
/// The non-negativity guarantee is the `@invariant` below, not prose — so it is
/// checked at every public call boundary and harvested into the doc's contract
/// view. This class is the classic Eiffel example, which is the point: contracts
/// are the specification, Phaldoc supplies only the intent around them.
@invariant(() => _balance >= 0)
class BankAccount {
  /// Add funds. The precondition and postcondition are the spec; this `///`
  /// only says *why* the method exists. Note the absence of a prose
  /// "amount must be positive" — that fact lives solely in `@requires`.
  /// @example
  /// const a = BankAccount.opened(100)
  /// a.deposit(50)                 // ok
  /// a.deposit(0 - 1)              // raises PreconditionError
  @requires(amount > 0)
  @ensures(_balance == old(_balance) + amount)
  deposit(_ amount) { _balance = _balance + amount }

  /// Withdraw funds, refusing to overdraw.
  ///
  /// `@throws` here documents the *domain* error the body raises; the
  /// PreconditionError from `@requires` is derived automatically (Phaldoc §8.3)
  /// and must not be written by hand.
  /// @throws InsufficientFunds — when `amount` exceeds the balance
  @requires(amount > 0)
  @ensures(_balance == old(_balance) - amount)
  withdraw(_ amount) {
    (amount > _balance).ifTrue { InsufficientFunds.raise("overdraw") }
    _balance = _balance - amount
  }

  /// The current balance. A pure query — no contract needed.
  balance { return _balance }
}

// ── Layout tier: field declarations + @construct + @get/@set ──────────────────

/// A 2-D point. `@construct` derives `new(x:, y:)` from the declared field order.
///
/// Field-declaration order **is** the constructor's calling convention
/// (Phaldoc §8.5 / annotations-construct.md): reordering `_x` and `_y` silently
/// changes `new(x:,y:)` and is a breaking API change. Documented, not routed around.
@construct
class Point {
  /// The horizontal coordinate. This `///` flows to both the `x` getter and the
  /// `x=(_)` setter that `@get`/`@set` derive.
  @get @set _x
  /// The vertical coordinate.
  @get @set _y

  /// Euclidean distance to another point.
  /// @param o — the other `Point`
  /// @returns the distance as a `Number`
  distanceTo(_ o) {
    const dx = _x - o.x
    const dy = _y - o.y
    return Math.sqrt(dx * dx + dy * dy)
  }
}

// ── Bridge A: @data / @sealed / @variant → algebraic data + exhaustive match ──

/// A closed family of shapes with structural equality and exhaustive matching.
///
/// `@data` derives `==`, `hash`, and `with(...)` (the equality ladder keeps
/// `==` and `hash` together — never a lone `==`). `@sealed` freezes the variant
/// set, which is the *sole* route to checkable `match` exhaustiveness in a
/// language with no type checker — so adding a `@variant` is a breaking change
/// for every `match` over `Shape`.
@data @sealed
class Shape {
  /// A circle of the given radius.
  @variant Circle(radius:)
  /// An axis-aligned rectangle.
  @variant Rect(w:, h:)

  /// Area of the shape, by exhaustive match over the sealed variant set.
  /// @returns the area as a `Number`
  area {
    return self.match {
      Circle(_ r)  => 3.14159 * r * r ;
      Rect(w, h) => w * h
    }
  }
}

// ── Bridge C: @observable / @computed → reactive dataflow ─────────────────────

/// A shopping cart whose `total` recomputes reactively as items change.
class Cart {
  /// The line items. `@observable` makes writes notify subscribers.
  @observable _items
  /// The running total. `@computed` memoizes and recomputes when `_items` goes
  /// dirty; the glitch policy (recompute once after inputs settle) is a
  /// documented semantic, not an accident.
  @computed total => _items.fold(0) { s, it => s + it.price }
}
