// Ported from wren/test/benchmark/fib.wren. Range `1..5` (inclusive) has no
// Phalcom parser production (DotDot lexed, never consumed) — replaced with a
// while-counter. System.clock unimplemented — time externally.
class Fib {
  static get(n) {
    if (n < 2) { return n }
    return Fib.get(n - 1) + Fib.get(n - 2)
  }
}

let i = 1
while (i <= 5) {
  System.print(Fib.get(28))
  i = i + 1
}
