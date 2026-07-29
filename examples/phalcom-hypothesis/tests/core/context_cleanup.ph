// Phase 02 — context installation is stack-disciplined and cleanup is
// guaranteed for normal returns and thrown errors.

import Assert from hypothesis
import context from "core/context"

const stack = context._PropertyContextStack.new()
const outer = context._PropertyContext.new()
const inner = context._PropertyContext.new()

const value = stack.with(outer) {
  Assert.equal(outer, stack.current.unwrap)
  return 42
}
Assert.equal(42, value)
Assert.isTrue(stack.current.isNone)

const outcome = {
  stack.with(outer) {
    stack.with(inner) {
      Assert.equal(inner, stack.current.unwrap)
      throw Error.new("expected")
    }
  }
}.attempt()

Assert.isTrue(outcome.isErr)
Assert.isTrue(stack.current.isNone)

System.print("PASS core context cleanup")
