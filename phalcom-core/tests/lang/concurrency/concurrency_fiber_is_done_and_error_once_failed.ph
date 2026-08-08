// area: concurrency
// spec: concurrency.md §1 (Interface table: `isDone`, `error`); U-FIBER-REFLECT
// status: PASS

// `Fiber#isDone` is `true` once the receiver's entry raised uncaught
// (`Failed`, captured via a resumer's `try`), and `Fiber#error` is
// `Some(e)` wrapping the *same* captured `Error` instance `try()` already
// delivered — not a re-wrapped copy (`Object#==` identity, unwrapped via
// `match`, since `Some`/`Option` have no custom `==` override).

const f = Fiber.new || { Error.new("boom").raise() }

System.print(f.error)
const e = f.try()
System.print(f.isDone)
System.print(f.error.match(some: |v| { v == e }, none: || { false }))
