// Release integration test: preserve the broad-v1 public behavior through
// direct aliases to authoritative modules. This fixture intentionally contains a
// falsifying property whose minimal counterexample is 10.

import Given from hypothesis
import Case from hypothesis
import Check from hypothesis
import CheckConfig from hypothesis
import Gen from hypothesis
import Property from hypothesis
import PropertySuite from hypothesis
import PropertyRunner from hypothesis
import PropertyReporter from hypothesis

class Arithmetic {
  @class
  abs(n: Int) -> Int {
    if n < 0 {
      return 0 - n
    }

    return n
  }
}

class ArithmeticProperties is PropertySuite {
  @Case(-1)
  @Case(0)
  @Case(1)
  @Given(Gen.int)
  absIsNonNegative(n: Int) {
    self.assertTrue(Arithmetic.abs(n) >= 0)
  }

  @Given(Gen.int)
  absIsIdempotent(n: Int) {
    self.assertEqual(
      Arithmetic.abs(n),
      Arithmetic.abs(Arithmetic.abs(n))
    )
  }

  @Given(Gen.int)
  nonZeroDividedByItselfIsOne(n: Int) {
    Property.assume(n != 0)
    self.assertEqual(1, n ~/ n)
  }

  @Check(
    CheckConfig.standard
      .examples(100)
      .seed(20260723)
  )
  @Given(Gen.int(min: 0, max: 1000))
  valuesStayBelowTen(n: Int) {
    self.assertTrue(n < 10)
  }
}

const config: CheckConfig =
  CheckConfig.standard
    .examples(100)
    .seed(20260723)

const run = PropertyRunner.run(
  [ArithmeticProperties],
  with: config
)

PropertyReporter.console.report(run)
