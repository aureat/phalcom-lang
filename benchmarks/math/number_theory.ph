// ============================================================
// number_theory — modular exponentiation, Fermat's little theorem, perfect
//                  numbers, Collatz, prime counting
// Benchmark corpus — NOT wired into CI. Run manually:
//   cargo run -p phalcom-core --bin phalcom -- benchmarks/math/number_theory.ph
// Tier 1 (pure): recursion + while loops + integer arithmetic via % and /.
// Verifies via theorems (self-checking, no magic decimals):
//   Fermat: a^(p-1) == 1 (mod p) for prime p, gcd(a,p)=1
//   perfect numbers: sigma(n) - n == n for 6 and 28
//   Collatz: known step counts for 6 and 27.
// Expected: every printed line is `true`.
// ============================================================

class NT {
  // Fast modular exponentiation: (base^e) mod m, keeping intermediates small.
  static modpow(base, e, m) {
    if (e == 0) { return 1 }
    const half = NT.modpow(base, (e - (e % 2)) / 2, m)
    const sq = (half * half) % m
    if (e % 2 == 1) { return (sq * base) % m }
    return sq
  }

  static isPrime(n) {
    if (n < 2) { return false }
    const d = 2
    while (d * d <= n) {
      if (n % d == 0) { return false }
      d = d + 1
    }
    return true
  }

  static countPrimesBelow(n) {
    const count = 0
    const k = 2
    while (k < n) {
      if (NT.isPrime(k)) { count = count + 1 }
      k = k + 1
    }
    return count
  }

  // Sum of proper divisors (aliquot sum). n is perfect iff this equals n.
  static aliquot(n) {
    const sum = 0
    const d = 1
    while (d < n) {
      if (n % d == 0) { sum = sum + d }
      d = d + 1
    }
    return sum
  }

  // Number of Collatz steps to reach 1.
  static collatz(start) {
    const n = start
    const steps = 0
    while (n > 1) {
      if (n % 2 == 0) { n = n / 2 } else { n = 3 * n + 1 }
      steps = steps + 1
    }
    return steps
  }
}

// --- modular exponentiation -----------------------------------------------
System.print(NT.modpow(2, 10, 1000) == 24)         // true (1024 mod 1000)
System.print(NT.modpow(7, 4, 10) == 1)             // true (2401 mod 10)
System.print(NT.modpow(3, 0, 7) == 1)              // true

// --- Fermat's little theorem: a^(p-1) == 1 (mod p) for prime p ------------
System.print(NT.modpow(2, 96, 97) == 1)            // true
System.print(NT.modpow(3, 96, 97) == 1)            // true
System.print(NT.modpow(5, 12, 13) == 1)            // true

// --- perfect numbers: aliquot(n) == n -------------------------------------
System.print(NT.aliquot(6) == 6)                   // true (1+2+3)
System.print(NT.aliquot(28) == 28)                 // true (1+2+4+7+14)
System.print(NT.aliquot(12) == 16)                 // true (abundant, not perfect)

// --- prime counting: pi(30) == 10 -----------------------------------------
System.print(NT.countPrimesBelow(30) == 10)        // true

// --- Collatz step counts (known values) -----------------------------------
System.print(NT.collatz(6) == 8)                   // true
System.print(NT.collatz(27) == 111)               // true
