// area: concurrency
// spec: concurrency.md; E008
// status: PASS
// A fiber admitted once remains queued until the scheduler owns its resume.
// Duplicate admission and a direct public resume must both fail without
// disturbing an unrelated healthy scheduled fiber.

const f = Fiber.new || { System.print("duplicate-ran") }
const probe = Fiber.new || { f.call() }
System.schedule(probe)
System.schedule(f)
const duplicate = || { System.schedule(f) }.on(Error) |e| { e.message }
System.print(duplicate)
System.schedule(|| { System.print("healthy-ran") })
System.runScheduled()
System.print(probe.error.unwrapOr(None).message)
