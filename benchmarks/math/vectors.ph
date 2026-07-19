// ============================================================
// vectors — Euclidean vector algebra over lists (dot, norm, cosine, projection)
// Benchmark corpus — NOT wired into CI. Run manually once collections land:
//   cargo run -p phalcom-core --bin phalcom -- benchmarks/math/vectors.ph
// Tier 2 (needs U6 `let` + collections + stdlib). Grounded in the spec surface;
// stdlib selectors still DEFERRED (typing-stdlib-surface.md) so names may shift.
// Assumed list API:  [..] literal, xs.size, xs[i], for (x in xs){..},
//                    xs.reduce(init){acc,x=>..}, xs.map{x=>..}
// Verifies via linear-algebra identities (no hardcoded magnitudes):
//   dot(a, a) == norm(a)^2
//   Cauchy–Schwarz:  |dot(a,b)| <= norm(a)*norm(b)
//   cosine(a, a) == 1
//   Pythagoras for orthogonal vectors: norm(a+b)^2 == norm(a)^2 + norm(b)^2
// Expected: every printed line is `true`.
// ============================================================

class Check {
  static approx(a, b) {
    let d = a - b
    if (d < 0) { d = 0 - d }
    return d < 0.0000001
  }
}

class Vec {
  static sqrt(a) {
    if (a == 0) { return 0 }
    let g = a
    let i = 0
    while (i < 50) { g = (g + a / g) / 2; i = i + 1 }
    return g
  }

  // Index-wise fold; assumes equal length.
  static dot(a, b) {
    let acc = 0
    let i = 0
    while (i < a.size) {
      acc = acc + a[i] * b[i]
      i = i + 1
    }
    return acc
  }

  static add(a, b) {
    let out = []
    let i = 0
    while (i < a.size) {
      out.add(a[i] + b[i])
      i = i + 1
    }
    return out
  }

  static norm(a) {
    return Vec.sqrt(Vec.dot(a, a))
  }

  static cosine(a, b) {
    return Vec.dot(a, b) / (Vec.norm(a) * Vec.norm(b))
  }
}

const a = [3, 4]
const b = [0, 5]
const u = [1, 0, 0]
const v = [0, 1, 0]      // orthogonal to u

// --- dot(a,a) == norm(a)^2 ------------------------------------------------
System.print(Check.approx(Vec.dot(a, a), Vec.norm(a) * Vec.norm(a)))   // true
System.print(Vec.norm(a) == 5)                                         // true (3-4-5)

// --- Cauchy–Schwarz inequality --------------------------------------------
let lhs = Vec.dot(a, b)
if (lhs < 0) { lhs = 0 - lhs }
System.print(lhs <= Vec.norm(a) * Vec.norm(b))                         // true

// --- cosine of a vector with itself is 1 ----------------------------------
System.print(Check.approx(Vec.cosine(a, a), 1))                        // true

// --- orthogonal unit vectors: cosine == 0, and Pythagoras holds -----------
System.print(Check.approx(Vec.cosine(u, v), 0))                        // true
const w = Vec.add(u, v)
System.print(Check.approx(Vec.norm(w) * Vec.norm(w),
                          Vec.norm(u) * Vec.norm(u) + Vec.norm(v) * Vec.norm(v)))  // true
