// ============================================================
// continued_fractions — fixed-point iteration of continued fractions
// Benchmark corpus — NOT wired into CI. Run manually:
//   cargo run -p phalcom-core --bin phalcom -- benchmarks/math/continued_fractions.ph
// Tier 1 (pure): static methods, while loops, fixed-point iteration.
// Verifies via the DEFINING identity of each constant (no known decimal needed):
//   golden ratio phi:   phi^2 == phi + 1
//   silver-ish sqrt(2): (y-1)^2 == 2, from the CF [1; 2, 2, 2, ...]
// Cross-check against an independent Newton sqrt.
// Expected: every printed line is `true`.
// ============================================================

class Check {
  @class
  approx(_ a, _ b) {
    const d = a - b
    if (d < 0) { d = 0 - d }
    return d < 0.0000001
  }
}

class CF {
  @class
  sqrt(_ a) {
    const g = a
    const i = 0
    while (i < 50) { g = (g + a / g) / 2; i = i + 1 }
    return g
  }

  // phi = 1 + 1/(1 + 1/(1 + ...)); iterate x = 1 + 1/x to the fixed point.
  @class
  golden {
    const x = 1
    const i = 0
    while (i < 90) { x = 1 + 1 / x; i = i + 1 }
    return x
  }

  // sqrt(2) via CF [1; 2,2,2,...]: iterate y = 2 + 1/y (-> 1 + sqrt 2), so sqrt2 = y - 1.
  @class
  rootTwo {
    const y = 2
    const i = 0
    while (i < 90) { y = 2 + 1 / y; i = i + 1 }
    return y - 1
  }
}

// --- golden ratio: defining identity phi^2 == phi + 1 ---------------------
const phi = CF.golden
System.print(Check.approx(phi * phi, phi + 1))                 // true  (pure identity)
System.print(Check.approx(phi, (1 + CF.sqrt(5)) / 2))          // true  (closed form)

// --- sqrt(2) from its continued fraction ----------------------------------
const r = CF.rootTwo
System.print(Check.approx(r * r, 2))                           // true  (pure identity)
System.print(Check.approx(r, CF.sqrt(2)))                      // true  (vs Newton)

// --- a reciprocal check: 1/phi == phi - 1 ---------------------------------
System.print(Check.approx(1 / phi, phi - 1))                   // true
