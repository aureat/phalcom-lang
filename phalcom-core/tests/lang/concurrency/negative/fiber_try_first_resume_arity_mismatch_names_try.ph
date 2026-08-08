// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: NEGATIVE
// Regression: `fiber_resume`'s first-resume arity error used to hardcode
// `signature: "call"` regardless of which resume primitive actually failed
// (fiber.rs). A first resume via `try(_)` with the wrong argument count must
// name "try", not "call", in the diagnostic.

const inner = Fiber.new |x| { x }
inner.try()
