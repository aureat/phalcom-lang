// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: NEGATIVE
// Ported from Wren `test/core/fiber/yield_with_value_from_main.wren`: the
// root fiber has no resumer, so `Fiber.yield(_)` at top level is illegal
// (fiber.rs `fiber_yield`) — the root-fiber counterpart to
// `fiber_abort_root_raises.ph`.

System.print("before")
Fiber.yield(1)
System.print("not reached")
