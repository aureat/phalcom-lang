// Phase 06: same error type at two assertion sites has distinct FailureOrigin.

import PropertyAssertionError from hypothesis
import Assert from hypothesis

const first = || {
  Assert.true(false)
}.attempt()

const second = || {
  Assert.false(true)
}.attempt()

first.match(
  ok: |_| { Assert.fail("first assertion should fail") },
  error: |firstError| {
    second.match(
      ok: |_| { Assert.fail("second assertion should fail") },
      error: |secondError| {
        Assert.true(firstError.isA(PropertyAssertionError))
        Assert.true(secondError.isA(PropertyAssertionError))
        Assert.false(
          firstError.failureOrigin.sameSite(secondError.failureOrigin)
        )
      }
    )
  }
)
