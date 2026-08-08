// ============================================================
// skynet — a million-fiber microbenchmark (atemerev/skynet), ported from Wren.
// Benchmark corpus — NOT wired into CI. Run directly:
//   cargo run -r -p phalcom-core --bin phalcom -- benchmarks/concurrency/skynet.ph
// Spawns 1,111,111 fibers in a 10-way fan-out tree of depth 6 (10^6 leaves),
// each leaf yielding its index, each interior node summing its children via
// `.call()`. Not self-verifying via identities like benchmarks/math/* — it
// asserts one fixed value: sum(0..999999) == 499999500000.
//
// Deviations from the original Wren source (docs/wren mirror), and why:
//   - `for (i in 0...div)` — Phalcom lexes `..`/`...` (DotDot/DotDotDot) but
//     the parser never wires them into an expression production, so a range
//     literal in `for`-position is a hard parse error. Replaced with an
//     explicit `while` counter loop. See forge finding: range-literal parser
//     wiring is dead code (tokens exist, no production consumes them).
//   - `%(expr)` string interpolation — Phalcom's surface is `\(expr)`
//     (ADR-0022), not Wren's `%()`. Direct syntax swap, not a gap.
//   - `System.clock` — unimplemented (pending fixture:
//     phalcom-core/tests/lang/system/pending/system_clock.ph). Timing done
//     externally (`time <bin> skynet.ph`) instead of in-language.
//
// Wren (release, real fibers):        ~0.57s  (bin/wren_test, this repo's mirror)
// Phalcom (release):                  ~16.6s  (~29x slower)
// Root cause: no inline caches + IndexMap (hashed) method dispatch on every
// send, vs Wren's dense symbol-indexed method array. See U-HOTPATH.
// ============================================================

class Skynet {
  @class
  makeFiber(_ num, _ size, _ div) {
    return Fiber.new {
      if (size == 1) {
        Fiber.yield(num)
      } else {
        let fibers = List.new()
        let i = 0
        while (i < div) {
          let subNum = num + i * (size / div)
          fibers.add(Skynet.makeFiber(subNum, size / div, div))
          i = i + 1
        }
        let sum = 0
        for (task in fibers) {
          sum = sum + task.call()
        }
        Fiber.yield(sum)
      }
    }
  }
}

let result = Skynet.makeFiber(0, 1000000, 10).call()
System.print("Result: \(result)")
