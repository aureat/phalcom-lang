// area: runtime-errors
// spec: blocks.md §5; object-model.md §4; ADR-0013 (frame-token non-local return)
// status: NEGATIVE
// A block that captures a `return` escapes its home method (`make` returns the
// block without calling it). Invoking it from a *separate* call site, after
// `make`'s activation is already gone, has no live home frame to unwind to —
// the frame-token generation compare fails and raises `DeadFrameError`
// (Smalltalk's `BlockCannotReturn`) rather than corrupting the stack.
class Maker {
  make() { return { return 1 } }
}
const escaped = Maker.new().make()
System.print(escaped.call())
