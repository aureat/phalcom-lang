//! rationals — exact rational arithmetic with a user-defined `Rational` class.
//!
//! Benchmark corpus, **not** wired into CI. Run manually:
//! ```sh
//! cargo run -p phalcom-core --bin phalcom -- benchmarks/math/rationals.ph
//! ```
//! Tier 1.5 (object model): classes, instance fields, operator methods
//! (`+ - * / ==`), static factories, gcd reduction — all observed working at U5
//! (`83c908a`). Grouped as its own tier because exact arithmetic makes `==`
//! assertions meaningful. Verifies exact identities (no float slop):
//! `1/2 + 1/3 + 1/6 == 1`, `2/4` reduces to `1/2`, and the telescoping sum
//! `Σ_{k=1..n} 1/(k(k+1)) == n/(n+1)`. Every printed line should be `true`.
//!
//! Doc comments below follow **Phaldoc** (`docs/spec/experimental/doc-comments-phaldoc.md`):
//! `//!` documents this file, `///` documents the item beneath it. The parser
//! ignores all of it — `///`/`//!` are `//`-prefixed trivia today.

/// An exact rational number `num/den`, always stored in lowest terms with a
/// positive denominator, so `==` is a structural equality.
///
/// Instances are canonical: `Rational.of(2, 4)` and `Rational.of(1, 2)` are `==`.
class Rational {
  /// Construct a reduced rational from a numerator and denominator.
  ///
  /// Allocates, then delegates to `init(_,_)` to sign-normalise and reduce.
  /// @param n — the numerator (any integer-valued Number)
  /// @param d — the denominator; its sign is moved onto the numerator
  /// @returns a `Rational` in lowest terms with `den > 0`
  /// @see #init(_,_)
  /// @example
  /// System.print(Rational.of(2, 4) == Rational.of(1, 2))   // true
  static of(n, d) {
    const r = self.new()
    r.init(n, d)
    return r
  }

  /// Sign-normalise and reduce this rational in place. Internal — prefer `of(_,_)`.
  /// @param n — the raw numerator
  /// @param d — the raw denominator (may be negative)
  init(n, d) {
    let nn = n
    let dd = d
    if (dd < 0) { nn = 0 - nn; dd = 0 - dd }   // keep denominator positive
    let g = Rational.gcd(Rational.absi(nn), dd)
    if (g == 0) { g = 1 }
    _num = nn / g
    _den = dd / g
  }

  /// @returns the numerator, carrying the sign of the rational.
  num { return _num }
  /// @returns the denominator, always positive.
  den { return _den }

  /// Sum. @param o — the addend `Rational`. @returns a reduced `Rational`.
  +(o) { return Rational.of(_num * o.den + o.num * _den, _den * o.den) }
  /// Difference. @param o — the subtrahend. @returns a reduced `Rational`.
  -(o) { return Rational.of(_num * o.den - o.num * _den, _den * o.den) }
  /// Product. @param o — the multiplicand. @returns a reduced `Rational`.
  *(o) { return Rational.of(_num * o.num, _den * o.den) }
  /// Quotient. @param o — the divisor. @returns a reduced `Rational`.
  /(o) { return Rational.of(_num * o.den, _den * o.num) }
  /// Structural equality — sound because both operands are kept canonical.
  /// @param o — the `Rational` to compare against. @returns a `Bool`.
  ==(o) { return _num == o.num and _den == o.den }

  /// Euclid's algorithm. @param a @param b — non-negative integers.
  /// @returns their greatest common divisor.
  static gcd(a, b) {
    if (b == 0) { return a }
    return Rational.gcd(b, a % b)
  }

  /// Integer absolute value. @param x — a Number. @returns `|x|`.
  static absi(x) {
    if (x < 0) { return 0 - x }
    return x
  }
}

// --- exact sum of unit fractions: 1/2 + 1/3 + 1/6 == 1/1 -------------------
const s = Rational.of(1, 2) + Rational.of(1, 3) + Rational.of(1, 6)
System.print(s == Rational.of(1, 1))                     // true

// --- reduction: 2/4 and 1/2 are the same rational -------------------------
System.print(Rational.of(2, 4) == Rational.of(1, 2))     // true
System.print(Rational.of(0 - 3, 0 - 6) == Rational.of(1, 2))  // true (sign normalized)

// --- ring identities ------------------------------------------------------
const half = Rational.of(1, 2)
const third = Rational.of(1, 3)
System.print(half * Rational.of(2, 1) == Rational.of(1, 1))       // true
System.print((half + third) - third == half)                      // true (additive inverse)
System.print(half / half == Rational.of(1, 1))                    // true

// --- telescoping sum: sum_{k=1..n} 1/(k(k+1)) == n/(n+1) -------------------
let acc = Rational.of(0, 1)
let k = 1
while (k <= 10) {
  acc = acc + Rational.of(1, k * (k + 1))
  k = k + 1
}
System.print(acc == Rational.of(10, 11))                 // true
