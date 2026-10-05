// Phase 06: the fluent builder delegates to the shared search kernel.

import Assert from hypothesis
import Property from hypothesis
import Gen from hypothesis
import Settings from hypothesis

const result = Property
  .given(Gen.int(min: 0, max: 100))
  .using(Settings.standard.examples(25).seed(20260723))
  .check |value| {
    Assert.true(value >= 0)
  }

Assert.true(result.passed)
