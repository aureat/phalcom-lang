// Phase 06: explicit @Case arguments retain reflected names and are not shrunk.

import Assert from hypothesis
import Given from hypothesis
import Case from hypothesis
import Gen from hypothesis
import PropertySuite from hypothesis
import PropertyRunner from hypothesis
import Settings from hypothesis

class ExplicitCaseProperties is PropertySuite {
  @Case(10, 2)
  @Given(Gen.int(min: 0, max: 100), Gen.int(min: 1, max: 10))
  quotientIsSmall(dividend: Int, divisor: Int) {
    self.assertTrue(dividend ~/ divisor < 5)
  }
}

const suite = PropertyRunner.run(
  const [ExplicitCaseProperties],
  with: Settings.standard.examples(1)
)
const property = suite.runs.at(0)
const named = property.namedArguments
Assert.equal(10, named.at(#dividend))
Assert.equal(2, named.at(#divisor))
Assert.true(property.explicitFailure)
