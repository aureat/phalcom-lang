// area: concurrency
// spec: concurrency.md; ADR-0030 §2
// status: NEGATIVE
// Ported from Wren `test/core/fiber/new_wrong_arg_type.wren`: `Fiber.new(_)`
// requires a `Block`/`Closure` entry (fiber.rs `fiber_new`); a non-function
// argument is a type error.

Fiber.new("not a function")
