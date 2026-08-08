//! numeric_core — sqrt, gcd/lcm, factorial, fast exponentiation, primality.
//!
//! Benchmark corpus, **not** wired into CI. Run once the tree builds:
//! ```sh
//! cargo run -p phalcom-core --bin phalcom -- benchmarks/math/numeric_core.ph
//! ```
//! Tier 1 (pure): classes, static methods, recursion, `if`/`while`,
//! `+ - * / % < <= == and or` — every construct observed working at `83c908a`
//! (U5). Unexecuted draft. Verifies **identities**, not hardcoded decimals:
//! `sqrt(x)² == x`, `gcd(a,b)·lcm(a,b) == a·b`, recursive `factorial` ==
//! iterative. Every printed line should be `true`.
//!
//! Documented with **Phaldoc** (`docs/spec/experimental/doc-comments-phaldoc.md`).

/// Float comparison helpers — floats never compare exactly, so tests go through here.
class Check {
  /// Absolute-error equality within a fixed tolerance.
  /// @param a — first Number
  /// @param b — second Number
  /// @returns a `Bool`: `true` iff `|a - b| < 1e-6`
  @class
  approx(_ a, _ b) {
    const d = a - b
    if (d < 0) { d = 0 - d }
    return d < 0.000001
  }
}

/// Pure static numeric routines. Stateless — every method is a `static`.
class Math {
  /// @param x — a Number. @returns `|x|`.
  @class
  abs(_ x) {
    if (x < 0) { return 0 - x }
    return x
  }

  /// Square root by Newton–Raphson: `x_{n+1} = (x_n + a/x_n) / 2` (quadratic
  /// convergence, 40 fixed iterations).
  /// @param a — a non-negative Number
  /// @returns an approximation of `√a` accurate to `Check.approx` tolerance
  /// @example
  /// System.print(Check.approx(Math.sqrt(2) * Math.sqrt(2), 2))   // true
  @class
  sqrt(_ a) {
    if (a == 0) { return 0 }
    const g = a
    const i = 0
    while (i < 40) {
      g = (g + a / g) / 2
      i = i + 1
    }
    return g
  }

  // Euclid's algorithm.
  @class
  gcd(_ a, _ b) {
    if (b == 0) { return a }
    return Math.gcd(b, a % b)
  }

  @class
  lcm(_ a, _ b) {
    return a * b / Math.gcd(a, b)
  }

  @class
  factIter(_ n) {
    const acc = 1
    const k = 2
    while (k <= n) {
      acc = acc * k
      k = k + 1
    }
    return acc
  }

  @class
  factRec(_ n) {
    if (n < 2) { return 1 }
    return n * Math.factRec(n - 1)
  }

  // Exponentiation by squaring: O(log e) multiplications.
  @class
  ipow(_ base, _ e) {
    if (e == 0) { return 1 }
    const half = Math.ipow(base, (e - (e % 2)) / 2)
    const sq = half * half
    if (e % 2 == 1) { return sq * base }
    return sq
  }

  // Trial division; O(sqrt n).
  @class
  isPrime(_ n) {
    if (n < 2) { return false }
    const d = 2
    while (d * d <= n) {
      if (n % d == 0) { return false }
      d = d + 1
    }
    return true
  }
}

// --- sqrt identity: sqrt(x)^2 == x -----------------------------------------
System.print(Check.approx(Math.sqrt(2) * Math.sqrt(2), 2))     // true
System.print(Check.approx(Math.sqrt(144), 12))                 // true
System.print(Check.approx(Math.sqrt(1000000), 1000))           // true

// --- gcd/lcm identity: gcd(a,b) * lcm(a,b) == a * b ------------------------
System.print(Math.gcd(48, 36) == 12)                           // true
System.print(Math.gcd(48, 36) * Math.lcm(48, 36) == 48 * 36)   // true
System.print(Math.gcd(17, 5) == 1)                             // true (coprime)

// --- factorial: recursive == iterative ------------------------------------
System.print(Math.factRec(6) == 720)                           // true
System.print(Math.factIter(10) == 3628800)                     // true
System.print(Math.factRec(10) == Math.factIter(10))            // true

// --- fast exponentiation --------------------------------------------------
System.print(Math.ipow(2, 10) == 1024)                         // true
System.print(Math.ipow(3, 4) == 81)                            // true
System.print(Math.ipow(7, 0) == 1)                             // true

// --- primality ------------------------------------------------------------
System.print(Math.isPrime(97))                                 // true
System.print(Math.isPrime(91) == false)                        // true  (91 = 7*13)
System.print(Math.isPrime(2))                                  // true
System.print(Math.isPrime(1) == false)                         // true
