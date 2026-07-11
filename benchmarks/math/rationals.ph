// ============================================================
// rationals — exact rational arithmetic with a user-defined Rational class
// Benchmark corpus — NOT wired into CI. Run manually:
//   cargo run -p phalcom-core --bin phalcom -- benchmarks/math/rationals.ph
// Tier 1.5 (object model): classes, instance fields, operator methods (+ * ==),
//   static factories, gcd reduction. No collections/var required — this mostly
//   exercises features observed working at U5 (see person3.ph / cons-list probe),
//   but is grouped here because exact-arithmetic identities make it a strong test.
// Verifies EXACT equalities (rationals are exact, so `==` is legitimate here):
//   1/2 + 1/3 + 1/6 == 1
//   2/4 reduces to 1/2
//   telescoping sum_{k=1}^{n} 1/(k(k+1)) == n/(n+1)
// Expected: every printed line is `true`.
// ============================================================

class Rational {
  // Factory: allocate, then normalize sign and reduce to lowest terms.
  static of(n, d) {
    let r = self.new()
    r.init(n, d)
    return r
  }

  init(n, d) {
    let nn = n
    let dd = d
    if (dd < 0) { nn = 0 - nn; dd = 0 - dd }   // keep denominator positive
    let g = Rational.gcd(Rational.absi(nn), dd)
    if (g == 0) { g = 1 }
    _num = nn / g
    _den = dd / g
  }

  num { return _num }
  den { return _den }

  +(o) { return Rational.of(_num * o.den + o.num * _den, _den * o.den) }
  -(o) { return Rational.of(_num * o.den - o.num * _den, _den * o.den) }
  *(o) { return Rational.of(_num * o.num, _den * o.den) }
  /(o) { return Rational.of(_num * o.den, _den * o.num) }
  ==(o) { return _num == o.num and _den == o.den }

  static gcd(a, b) {
    if (b == 0) { return a }
    return Rational.gcd(b, a % b)
  }

  static absi(x) {
    if (x < 0) { return 0 - x }
    return x
  }
}

// --- exact sum of unit fractions: 1/2 + 1/3 + 1/6 == 1/1 -------------------
let s = Rational.of(1, 2) + Rational.of(1, 3) + Rational.of(1, 6)
System.print(s == Rational.of(1, 1))                     // true

// --- reduction: 2/4 and 1/2 are the same rational -------------------------
System.print(Rational.of(2, 4) == Rational.of(1, 2))     // true
System.print(Rational.of(0 - 3, 0 - 6) == Rational.of(1, 2))  // true (sign normalized)

// --- ring identities ------------------------------------------------------
let half = Rational.of(1, 2)
let third = Rational.of(1, 3)
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
