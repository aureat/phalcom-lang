// Phase 06: GivenArgs overrides bind by reflected parameter name.

import Assert from hypothesis
import Given from hypothesis
import GivenArgs from hypothesis
import Gen from hypothesis
import PropertySuite from hypothesis
import PropertyRunner from hypothesis
import Settings from hypothesis

class OverrideProperties is PropertySuite {
  @Given(
    GivenArgs.new()
      .for(#count, use: Gen.int(min: 1, max: 3))
  )
  boundedCount(count: Int, values: List<Int>) {
    self.assertTrue(count >= 1 and count <= 3)
    self.assertTrue(values.isA(List))
  }
}

const run = PropertyRunner.run(
  const [OverrideProperties],
  with: Settings.standard.examples(10)
)
Assert.equal(1, run.passedCount)
