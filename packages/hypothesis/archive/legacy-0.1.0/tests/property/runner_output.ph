// Phase 06 acceptance: reflective runner returns canonical property identities.

import Assert from hypothesis
import Given from hypothesis
import PropertySuite from hypothesis
import PropertyRunner from hypothesis
import Settings from hypothesis

class ArithmeticProperties is PropertySuite {
  @Given
  additionIsCommutative(left: Int, right: Int) {
    self.assertEqual(left + right, right + left)
  }
}

class CollectionProperties is PropertySuite {
  @Given
  reverseTwice(values: List<Int>) {
    self.assertEqual(values, values.reverse.reverse)
  }
}

const suite = PropertyRunner.run(
  const [ArithmeticProperties, CollectionProperties],
  with: Settings.standard.examples(25)
)

Assert.equal(2, suite.passedCount)
Assert.equal(0, suite.failedCount)
Assert.equal(
  const [
    "PASS ArithmeticProperties.additionIsCommutative",
    "PASS CollectionProperties.reverseTwice",
    "",
    "2 passed, 0 failed"
  ],
  suite.summaryLines
)
