// Executable Phase 01 example using the temporary compatibility runner.

import Given from hypothesis
import CheckConfig from hypothesis
import Gen from hypothesis
import PropertySuite from hypothesis
import PropertyRunner from hypothesis
import PropertyReporter from hypothesis

class ArithmeticProperties is PropertySuite {
  @Given(Gen.int, Gen.int)
  additionIsCommutative(a: Int, b: Int) {
    self.assertEqual(a + b, b + a)
  }
}

const run = PropertyRunner.run(
  [ArithmeticProperties],
  with: CheckConfig.standard.seed(20260723)
)

PropertyReporter.console.report(run)
