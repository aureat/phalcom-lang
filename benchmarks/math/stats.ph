// ============================================================
// stats — descriptive statistics over a list (mean, variance, stddev, median)
// Benchmark corpus — NOT wired into CI. Run manually once collections land:
//   cargo run -p phalcom-core --bin phalcom -- benchmarks/math/stats.ph
// Tier 2 (needs U6 `var` + collections + stdlib). Grounded in the spec surface
// (lexical-structure.md §4/§6, control-flow.md §1, iteration-protocol.md), but the
// stdlib method surface is still DEFERRED (typing-stdlib-surface.md), so the exact
// selector names below may shift. Assumed list API:
//   [a, b, c]           list literal
//   xs.length           element count
//   xs[i]               indexing (returns element; see numeric-and-string-indexing.md)
//   for (x in xs) {..}  === xs.each { x => .. }
//   xs.reduce(init) { acc, x => .. }   left fold
//   xs.sorted           ascending copy
// Verifies via identities (independent of hardcoded results):
//   sum via reduce == sum via for-loop
//   two-pass variance == E[x^2] - E[x]^2
//   stddev^2 == variance,  variance >= 0.
// Expected: every printed line is `true`.
// ============================================================

class Check {
  static approx(a, b) {
    var d = a - b
    if (d < 0) { d = 0 - d }
    return d < 0.0000001
  }
}

class Stats {
  static sum(xs) {
    return xs.reduce(0) { acc, x => acc + x }
  }

  static mean(xs) {
    return Stats.sum(xs) / xs.length
  }

  // Two-pass variance: (1/n) * sum (x - mean)^2
  static variance(xs) {
    let m = Stats.mean(xs)
    var acc = 0
    for (x in xs) {
      acc = acc + (x - m) * (x - m)
    }
    return acc / xs.length
  }

  // Alternate route: E[x^2] - (E[x])^2. Must match variance() up to rounding.
  static varianceMoment(xs) {
    let m = Stats.mean(xs)
    var sq = 0
    for (x in xs) { sq = sq + x * x }
    return sq / xs.length - m * m
  }

  static median(xs) {
    let s = xs.sorted
    let n = s.length
    if (n % 2 == 1) {
      return s[(n - 1) / 2]
    }
    return (s[n / 2 - 1] + s[n / 2]) / 2
  }
}

let data = [4, 8, 15, 16, 23, 42]   // sum 108, n 6, mean 18

// --- two routes to the sum agree ------------------------------------------
var loopSum = 0
for (x in data) { loopSum = loopSum + x }
System.print(Stats.sum(data) == loopSum)                       // true
System.print(Stats.mean(data) == 18)                           // true

// --- two variance formulas agree, and stddev^2 == variance ----------------
System.print(Check.approx(Stats.variance(data), Stats.varianceMoment(data)))  // true
System.print(Stats.variance(data) >= 0)                        // true

// --- median of an even-length sample --------------------------------------
System.print(Stats.median(data) == (15 + 16) / 2)              // true  (== 15.5)

// --- shifting all data by c shifts the mean by c, variance unchanged ------
let shifted = data.map { x => x + 100 }
System.print(Stats.mean(shifted) == Stats.mean(data) + 100)                    // true
System.print(Check.approx(Stats.variance(shifted), Stats.variance(data)))      // true
