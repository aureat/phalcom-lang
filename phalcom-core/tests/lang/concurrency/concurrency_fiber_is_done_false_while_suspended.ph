// area: concurrency
// spec: concurrency.md §1 (Interface table: `isDone`); U-FIBER-REFLECT
// status: PASS

// `Fiber#isDone` is `false` while the receiver is mid-generator (yielded,
// not finished) — only `Done`/`Failed` flip it `true`.

const f = Fiber.new {
  Fiber.yield(1)
  2
}

f.call()
System.print(f.isDone)
