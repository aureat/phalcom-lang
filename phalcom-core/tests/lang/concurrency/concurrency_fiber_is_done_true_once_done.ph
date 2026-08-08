// area: concurrency
// spec: concurrency.md §1 (Interface table: `isDone`, `error`); U-FIBER-REFLECT
// status: PASS

// `Fiber#isDone` is `true` once the receiver's entry has returned cleanly
// (`Done`), and `Fiber#error` stays `None` — `result` holds the return
// value, not an `Error`; `error` must not conflate the two.

const f = Fiber.new || { 42 }

f.call()
System.print(f.isDone)
System.print(f.error)
