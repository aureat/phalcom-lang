// area: concurrency
// spec: concurrency.md; ADR-0030 §6
// status: PASS
// C-FIB-4 adversarial: after `Fiber.abort(_)` delivers its payload to a
// `try()` resumer, the fiber is left `Failed` — an *un*-recoverable terminal
// state, same as `Done`. A second resume attempt (routed through a nested
// driver fiber so the failure is caught rather than escaping uncaught to the
// top level) raises the identical `NotAllowed` ("cannot resume a finished
// fiber") as resuming a completed fiber, confirming `Failed` and `Done` share
// one "cannot resume a finished fiber" guard (fiber.rs `fiber_resume`).
const f = Fiber.new || {
  Fiber.abort(Error.new())
}
const r1 = f.try()
System.print(r1.class.name)

const driver = Fiber.new || {
  f.call()
}
const r2 = driver.try()
System.print(r2.class.name)
System.print(r2.message)
