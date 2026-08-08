// area: iteration
// spec: iteration.md; ADR-0035; concurrency.md; ADR-0030
// status: PASS
// C-ITER-8: a `for` loop body running inside a `Fiber` suspends at each
// `Fiber.yield` and resumes at the next cursor position on the next `call` —
// proof that the direct-jump `for` lowering (C-ITER-4, no `block_call` on the
// taken path) composes with fiber suspension instead of yielding across a
// native frame.

const f = Fiber.new || {
  for (x in [1, 2, 3]) { Fiber.yield(x) }
}
System.print(f.call())
System.print(f.call())
System.print(f.call())
