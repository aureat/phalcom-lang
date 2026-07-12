// area: concurrency
// spec: concurrency.md §4 (impl-spec); ADR-0013 (frame-token non-local return); ADR-0030
// status: NEGATIVE
// C-FIB-5: a block that captures a `return` escapes its home method, then
// runs on a *different* fiber's stack than the one that created it. By the
// time that fiber's `call()` actually executes the block, `make()`'s home
// activation is long gone — the frame-token generation compare fails and
// raises `DeadFrameError`, exactly as the intra-fiber escaping-block case
// does, proving the fencing is fiber-agnostic (ADR-0013).

class Maker {
  make() { return { return 1 } }
}
let escaped = Maker.new().make()
let f = Fiber.new(escaped)
f.call()
