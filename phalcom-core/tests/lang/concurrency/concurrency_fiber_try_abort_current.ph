// area: concurrency
// spec: concurrency.md; ADR-0030 §6
// status: PASS
// C-FIB-4: `Fiber#try` captures an uncaught `Fiber.abort` as the delivered
// `Error` value rather than propagating past the fiber boundary; `Fiber.current`
// answers the fiber it is sent from.

const failing = Fiber.new || {
  Fiber.abort(Error.new())
}
const result = failing.try()
System.print(result.class.name)

const g = Fiber.new || {
  System.print(Fiber.current==g)
}
g.call()
