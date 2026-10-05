// Phase 06: an unannotated parameter is rejected before generation.

import Assert from hypothesis
import StrategyResolutionError from hypothesis
import Given from hypothesis
import PropertySuite from hypothesis
import PropertyRunner from hypothesis
import Settings from hypothesis

class MissingAnnotationProperties is PropertySuite {
  @Given
  missing(value) {
    self.assertTrue(value != None)
  }
}

const outcome = || {
  PropertyRunner.run(
    const [MissingAnnotationProperties],
    with: Settings.standard
  )
}.attempt()

outcome.match(
  ok: |_| { Assert.fail("expected strategy inference to fail") },
  error: |error| {
    Assert.true(error.isA(StrategyResolutionError))
    Assert.true(error.message.unwrap.includes("parameter 'value'"))
    Assert.true(error.message.unwrap.includes("MissingAnnotationProperties.missing"))
  }
)
