// Phase 06: explicit @Given strategies have exact reflected arity.

import Assert from hypothesis
import PropertyDiscoveryError from hypothesis
import Given from hypothesis
import Gen from hypothesis
import PropertySuite from hypothesis
import PropertyRunner from hypothesis
import Settings from hypothesis

class ArityProperties is PropertySuite {
  @Given(Gen.int)
  mismatched(left: Int, right: Int) {
    self.assertEqual(left, right)
  }
}

const outcome = {
  PropertyRunner.run(const [ArityProperties], with: Settings.standard)
}.attempt()

outcome.match(
  ok: |_| { Assert.fail("expected explicit strategy arity failure") },
  error: |error| {
    Assert.true(error.isA(PropertyDiscoveryError))
    Assert.true(error.message.unwrap.includes("expected 2 strategies, received 1"))
  }
)
