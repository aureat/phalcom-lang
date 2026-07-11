// ============================================================
// monte_carlo — stochastic estimation with a pure-Phalcom PRNG
// Benchmark corpus — NOT wired into CI. Run manually:
//   cargo run -p phalcom-core --bin phalcom -- benchmarks/math/monte_carlo.ph
// Tier 1 (pure): Phalcom has NO random primitive, so the generator is written in
//   the language itself — a MINSTD / Park–Miller LCG (state = 16807*state mod
//   2^31-1). Every product stays below 2^45 < 2^53, so it is EXACT in f64 and the
//   whole simulation is deterministic and reproducible (fixed seed => fixed answer).
//   Needs only: classes, instance-field mutation, while loops, blocks. U5-era.
// Verifies via convergence to known values (statistics, not decimals):
//   law of large numbers: sample mean -> 1/2, sample variance -> 1/12
//   pi by 2D darts (quarter circle) and by 3D ball fraction (pi/6)
//   e as the expected count of uniforms whose running sum first exceeds 1
//   Monte Carlo integration of a block-valued f against its exact integral.
// Tolerances are generous (this is a *statistical* test); expected: all `true`.
// ============================================================

class Check {
  static within(a, b, tol) {
    let d = a - b
    if (d < 0) { d = 0 - d }
    return d < tol
  }
}

// MINSTD (Park–Miller) linear congruential generator.
class Rng {
  static seed(s) {
    let r = self.new()
    r.setState(s)
    return r
  }
  setState(s) { _state = s }

  // Advance and return the raw integer in [1, 2^31 - 2].
  nextInt {
    _state = (16807 * _state) % 2147483647
    return _state
  }

  // Uniform double in (0, 1).
  uniform {
    return self.nextInt / 2147483647
  }
}

class MC {
  // Law of large numbers: mean of n uniforms.
  static sampleMean(rng, n) {
    let acc = 0
    let i = 0
    while (i < n) { acc = acc + rng.uniform; i = i + 1 }
    return acc / n
  }

  // Population variance of n uniforms via E[x^2] - E[x]^2.
  static sampleVariance(rng, n) {
    let sum = 0
    let sumSq = 0
    let i = 0
    while (i < n) {
      let u = rng.uniform
      sum = sum + u
      sumSq = sumSq + u * u
      i = i + 1
    }
    let m = sum / n
    return sumSq / n - m * m
  }

  // pi/4 = P[(x,y) in unit quarter-disk], so pi ~ 4 * inside/n.
  static piDarts(rng, n) {
    let inside = 0
    let i = 0
    while (i < n) {
      let x = rng.uniform
      let y = rng.uniform
      if (x * x + y * y <= 1) { inside = inside + 1 }
      i = i + 1
    }
    return 4 * inside / n
  }

  // Volume of unit ball / volume of [0,1]^3 cube octant = pi/6, so pi ~ 6*inside/n.
  static piBall(rng, n) {
    let inside = 0
    let i = 0
    while (i < n) {
      let x = rng.uniform
      let y = rng.uniform
      let z = rng.uniform
      if (x * x + y * y + z * z <= 1) { inside = inside + 1 }
      i = i + 1
    }
    return 6 * inside / n
  }

  // Expected number of uniforms drawn until the running sum first exceeds 1 is e.
  static eEstimate(rng, trials) {
    let total = 0
    let t = 0
    while (t < trials) {
      let sum = 0
      let count = 0
      while (sum <= 1) {
        sum = sum + rng.uniform
        count = count + 1
      }
      total = total + count
      t = t + 1
    }
    return total / trials
  }

  // Monte Carlo integral of block f over [0,1]: mean of f at uniform samples.
  static integrate(rng, f, n) {
    let acc = 0
    let i = 0
    while (i < n) { acc = acc + f.call(rng.uniform); i = i + 1 }
    return acc / n
  }
}

// --- law of large numbers -------------------------------------------------
let g1 = Rng.seed(42)
System.print(Check.within(MC.sampleMean(g1, 200000), 0.5, 0.01))          // true
let g2 = Rng.seed(7)
System.print(Check.within(MC.sampleVariance(g2, 200000), 1 / 12, 0.005))  // true (1/12 ~ 0.0833)

// --- pi by two independent geometric estimators ---------------------------
let g3 = Rng.seed(2024)
System.print(Check.within(MC.piDarts(g3, 300000), 3.141592653589793, 0.02))  // true
let g4 = Rng.seed(99)
System.print(Check.within(MC.piBall(g4, 300000), 3.141592653589793, 0.03))   // true

// --- e via expected samples-to-exceed-1 -----------------------------------
let g5 = Rng.seed(123)
System.print(Check.within(MC.eEstimate(g5, 200000), 2.718281828459045, 0.02))  // true

// --- Monte Carlo integration cross-checked against exact integrals --------
let g6 = Rng.seed(555)
System.print(Check.within(MC.integrate(g6, x => 3 * x * x, 300000), 1, 0.02))  // true (int 3x^2 = 1)
let g7 = Rng.seed(777)
System.print(Check.within(MC.integrate(g7, x => 4 / (1 + x * x), 300000),
                          3.141592653589793, 0.02))                            // true (-> pi)
