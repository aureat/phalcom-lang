// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: PASS
// C-FIB adversarial: a fiber accumulates state across three separate
// `call(_)` resumes (not just echoing the argument back — each yielded value
// folds the prior delivered argument in), then completes by falling off the
// end of its body (no explicit `return` — a bare `return` inside a fiber's
// block entry is a non-local return with no live home frame and raises
// `DeadFrameError`, out of scope here). The completion value is delivered on
// the *next* `call`, same as any yield. Resuming the now-`Done` fiber from a
// *nested* driver fiber (so the failure is reachable via `try()` rather than
// escaping uncaught to the top level) raises `NotAllowed`
// ("cannot resume a finished fiber"), pinned via its caught class/message.
const acc = Fiber.new || {
  let total = 0
  total = total + Fiber.yield(total)
  total = total + Fiber.yield(total)
  total = total + Fiber.yield(total)
  "settled:" + total.toString
}
System.print(acc.call())
System.print(acc.call(1))
System.print(acc.call(2))
System.print(acc.call(3))
const driver = Fiber.new || {
  acc.call(99)
}
const r = driver.try()
System.print(r.class.name)
System.print(r.message)
