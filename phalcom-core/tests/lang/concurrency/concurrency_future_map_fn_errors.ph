// area: concurrency
// spec: concurrency.md §2; ADR-0030
// status: PASS
// U-FUTURE Slice A adversarial: `map(_)`/`then(_)` fire their block
// *synchronously* on an already-settled fulfilled receiver (plan §6.2) —
// if that block itself errors (here, a genuine `doesNotUnderstand` miss),
// the error propagates straight out of `map`/`then` as an ordinary raise,
// not wrapped into a rejected future. Driven through a fiber so the
// otherwise-uncaught raise is captured via `try()` instead of escaping to
// the top level.
const f = Fiber.new || {
  Future.value(1).map |v| { v.frobnicate() }
}
const r = f.try()
System.print(r.class.name)

const g = Fiber.new || {
  Future.value(1).then |v| { v.frobnicate() }
}
const r2 = g.try()
System.print(r2.class.name)
