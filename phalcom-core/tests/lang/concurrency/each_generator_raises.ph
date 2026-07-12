// area: concurrency
// spec: concurrency.md; ADR-0030 §4; iteration.md
// status: PASS
// `Fiber.yield` reached through `List#each`'s native block-call frame (not
// the compiler's direct-jump `for` lowering) raises
// `CannotYieldAcrossNativeFrame` — `each` never gets the C-ITER-4 preclusion
// guard `for` gets, since it dispatches through the ordinary
// `Function#call` native frame.

let f = Fiber.new {
  [1, 2, 3].each { x => Fiber.yield(x) }
}
let result = f.try()
System.print(result.class.name)
