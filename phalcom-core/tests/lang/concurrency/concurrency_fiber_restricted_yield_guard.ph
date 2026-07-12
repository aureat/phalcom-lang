// area: concurrency
// spec: concurrency.md; ADR-0030 §4
// status: PASS
// C-FIB-3: `Fiber.yield` under a re-entrant native call frame (a block
// called via `Function#call`, not the sacred-selector-inlined `whileTrue`)
// raises `CannotYieldAcrossNativeFrame` instead of corrupting the fiber's
// suspended position.

let f = Fiber.new {
  let inner = { Fiber.yield(1) }
  inner.call()
}
let result = f.try()
System.print(result.class.name)
