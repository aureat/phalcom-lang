// ============================================================
// integration — Simpson's rule with block-valued integrands
// Benchmark corpus — NOT wired into CI. Run manually:
//   cargo run -p phalcom-core --bin phalcom -- benchmarks/math/integration.ph
// Tier 1 (pure): first-class blocks passed as arguments and invoked with .call,
//   closures capturing nothing, numeric while loops. This is the flagship test of
//   "functions as values": the integrator is generic over any f: Number -> Number.
// Verifies:
//   integral_0^1 4/(1+x^2) dx == pi     (a second, independent route to pi)
//   integral_0^1 3x^2 dx == 1,  integral_0^2 x^3 dx == 4  (Simpson is exact on cubics)
// Expected: every printed line is `true`.
// ============================================================

class Check {
  static approx(a, b) {
    const d = a - b
    if (d < 0) { d = 0 - d }
    return d < 0.00001
  }
}

class Simpson {
  // Composite Simpson's rule over [a, b] with n (even) subintervals:
  //   h/3 * [ f(x0) + 4 f(x1) + 2 f(x2) + 4 f(x3) + ... + f(xn) ]
  // `f` is any block taking one number and returning a number.
  static integrate(f, a, b, n) {
    const h = (b - a) / n
    const s = f.call(a) + f.call(b)
    const k = 1
    const x = 0
    const coef = 0
    while (k < n) {
      x = a + k * h
      coef = 4
      if (k % 2 == 0) { coef = 2 }
      s = s + coef * f.call(x)
      k = k + 1
    }
    return s * h / 3
  }
}

// --- a rational integrand whose exact integral is pi ----------------------
System.print(Check.approx(Simpson.integrate(x => 4 / (1 + x * x), 0, 1, 1000),
                          3.141592653589793))                 // true

// --- polynomials: Simpson is exact for degree <= 3 ------------------------
System.print(Check.approx(Simpson.integrate(x => 3 * x * x, 0, 1, 100), 1))      // true
System.print(Check.approx(Simpson.integrate(x => x, 0, 2, 100), 2))              // true
System.print(Check.approx(Simpson.integrate(x => x * x * x, 0, 2, 100), 4))      // true (2^4/4)
System.print(Check.approx(Simpson.integrate(x => x * x, 0, 1, 1000), 1 / 3))     // true

// --- linearity: integral(f) + integral(g) == integral(f+g) ----------------
const a = Simpson.integrate(x => x * x, 0, 1, 500)
const b = Simpson.integrate(x => x, 0, 1, 500)
const ab = Simpson.integrate(x => x * x + x, 0, 1, 500)
System.print(Check.approx(a + b, ab))                          // true
