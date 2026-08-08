// ============================================================
// transcendental — exp, sin, cos, atan via Taylor series; pi via Machin
// Benchmark corpus — NOT wired into CI. Run manually:
//   cargo run -p phalcom-core --bin phalcom -- benchmarks/math/transcendental.ph
// Tier 1 (pure): static methods, while loops, floating-point accumulation.
// Verifies via identities (the strongest tests need no known constant):
//   sin^2(x) + cos^2(x) == 1      (Pythagorean identity)
//   exp(a) * exp(b) == exp(a+b)   (exponential functional equation)
//   Machin's pi cross-checked against sin(pi/6) == 1/2.
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

class Fn {
  // exp(x) = sum_{k>=0} x^k / k!   (term_k = term_{k-1} * x / k)
  @class
  exp(_ x) {
    const term = 1
    const sum = 1
    const k = 1
    while (k <= 40) {
      term = term * x / k
      sum = sum + term
      k = k + 1
    }
    return sum
  }

  // sin(x) = x - x^3/3! + x^5/5! - ...
  @class
  sin(_ x) {
    const term = x
    const sum = x
    const k = 1
    while (k <= 30) {
      term = term * (0 - x * x) / ((2 * k) * (2 * k + 1))
      sum = sum + term
      k = k + 1
    }
    return sum
  }

  // cos(x) = 1 - x^2/2! + x^4/4! - ...
  @class
  cos(_ x) {
    const term = 1
    const sum = 1
    const k = 1
    while (k <= 30) {
      term = term * (0 - x * x) / ((2 * k - 1) * (2 * k))
      sum = sum + term
      k = k + 1
    }
    return sum
  }

  // atan(x) = x - x^3/3 + x^5/5 - ...   (converges for |x| < 1)
  @class
  atan(_ x) {
    const power = x
    const sum = x
    const k = 1
    while (k <= 60) {
      power = power * (0 - x * x)
      sum = sum + power / (2 * k + 1)
      k = k + 1
    }
    return sum
  }

  // Machin's formula: pi = 16*atan(1/5) - 4*atan(1/239). Fast convergence.
  @class
  pi {
    return 16 * Fn.atan(1 / 5) - 4 * Fn.atan(1 / 239)
  }
}

// --- Pythagorean identity: sin^2 + cos^2 == 1 (pure, no constant) ----------
System.print(Check.approx(Fn.sin(0.5) * Fn.sin(0.5) + Fn.cos(0.5) * Fn.cos(0.5), 1))  // true
System.print(Check.approx(Fn.sin(1.0) * Fn.sin(1.0) + Fn.cos(1.0) * Fn.cos(1.0), 1))  // true
System.print(Check.approx(Fn.sin(2.0) * Fn.sin(2.0) + Fn.cos(2.0) * Fn.cos(2.0), 1))  // true

// --- exponential functional equation: exp(a)*exp(b) == exp(a+b) ------------
System.print(Check.approx(Fn.exp(0.7) * Fn.exp(1.1), Fn.exp(1.8)))   // true
System.print(Check.approx(Fn.exp(0), 1))                             // true
System.print(Check.approx(Fn.exp(2) * Fn.exp(0 - 2), 1))             // true (exp(x)*exp(-x)=1)

// --- pi via Machin, then cross-check with sin(pi/6) == 1/2 -----------------
System.print(Check.approx(Fn.pi, 3.141592653589793))                // true
System.print(Check.approx(Fn.sin(Fn.pi / 6), 0.5))                  // true
System.print(Check.approx(Fn.cos(Fn.pi / 3), 0.5))                  // true
