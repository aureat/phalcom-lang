// ============================================================
// random_walk — stochastic simulation of random walks and diffusion laws
// Benchmark corpus — NOT wired into CI. Run manually:
//   cargo run -p phalcom-core --bin phalcom -- benchmarks/math/random_walk.ph
// Tier 1 (pure): reuses the in-language MINSTD PRNG (see monte_carlo.ph). Needs
//   only classes, instance-field mutation, while loops. U5-era.
// Verifies via probability theorems (self-checking):
//   1D symmetric walk: E[S_n] -> 0, and E[S_n^2] -> n  (variance grows linearly)
//   2D walk: mean-square displacement E[|R_n|^2] -> n  (diffusive scaling)
//   fair coin from the PRNG: P[heads] -> 1/2
// Tolerances are generous (statistical test); expected: every line is `true`.
// ============================================================

class Check {
  @class
  within(_ a, _ b, _ tol) {
    const d = a - b
    if (d < 0) { d = 0 - d }
    return d < tol
  }
  // relative tolerance for quantities that scale with n
  @class
  withinRel(_ a, _ b, _ rel) {
    const d = a - b
    if (d < 0) { d = 0 - d }
    const scale = b
    if (scale < 0) { scale = 0 - scale }
    return d < rel * scale
  }
}

class Rng {
  @class
  seed(_ s) {
    const r = self.new()
    r.setState(s)
    return r
  }
  setState(_ s) { _state = s }
  nextInt {
    _state = (16807 * _state) % 2147483647
    return _state
  }
  uniform { return self.nextInt / 2147483647 }
  // Fair Bernoulli step: +1 or -1 with equal probability.
  step { if (self.uniform < 0.5) { return 0 - 1 } return 1 }
}

class Walk {
  // Fraction of heads over n fair coin flips.
  @class
  headRate(_ rng, _ n) {
    const heads = 0
    const i = 0
    while (i < n) {
      if (rng.step == 1) { heads = heads + 1 }
      i = i + 1
    }
    return heads / n
  }

  // Average final position over `trials` 1D walks of `steps` each. -> 0.
  @class
  meanEndpoint(_ rng, _ trials, _ steps) {
    const total = 0
    const t = 0
    while (t < trials) {
      const pos = 0
      const s = 0
      while (s < steps) { pos = pos + rng.step; s = s + 1 }
      total = total + pos
      t = t + 1
    }
    return total / trials
  }

  // Mean of S_n^2 over `trials` 1D walks of `steps` each. -> steps.
  @class
  meanSquare1D(_ rng, _ trials, _ steps) {
    const total = 0
    const t = 0
    while (t < trials) {
      const pos = 0
      const s = 0
      while (s < steps) { pos = pos + rng.step; s = s + 1 }
      total = total + pos * pos
      t = t + 1
    }
    return total / trials
  }

  // Mean squared displacement of a 2D walk (independent x,y steps). -> 2*steps.
  @class
  meanSquare2D(_ rng, _ trials, _ steps) {
    const total = 0
    const t = 0
    while (t < trials) {
      const x = 0
      const y = 0
      const s = 0
      while (s < steps) {
        x = x + rng.step
        y = y + rng.step
        s = s + 1
      }
      total = total + x * x + y * y
      t = t + 1
    }
    return total / trials
  }
}

// --- fair coin: P[heads] -> 1/2 -------------------------------------------
const g0 = Rng.seed(42)
System.print(Check.within(Walk.headRate(g0, 200000), 0.5, 0.01))              // true

// --- 1D walk is unbiased: E[S_n] -> 0 -------------------------------------
const g1 = Rng.seed(2024)
System.print(Check.within(Walk.meanEndpoint(g1, 40000, 100), 0, 0.5))         // true

// --- 1D diffusion: E[S_n^2] -> n (variance grows linearly with step count) -
const g2 = Rng.seed(99)
System.print(Check.withinRel(Walk.meanSquare1D(g2, 40000, 100), 100, 0.05))   // true

// --- 2D diffusion: E[|R_n|^2] -> 2n ---------------------------------------
const g3 = Rng.seed(555)
System.print(Check.withinRel(Walk.meanSquare2D(g3, 40000, 100), 200, 0.05))   // true
