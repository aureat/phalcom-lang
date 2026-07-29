// Hypothesis for Phalcom — broad-v1 migration acceptance fixture
//
// The original prototype vocabulary remains available as direct aliases to the
// authoritative release modules.

import { Given, Case, Check, CheckConfig, Gen, Property, PropertySuite, PropertyRunner, PropertyReporter } from "hypothesis"
//
// The final property is intentionally false. Its purpose is to specify that the
// engine finds a failure, shrinks it, reports the minimal counterexample, keeps
// running the remaining suite machinery, and returns a failing run result.

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

  // This property is deliberately wrong.
  //
  // The generated domain begins at zero, so a structural integer shrinker must
  // reduce any failing value to 10: values 0 through 9 pass, and 10 is the
  // smallest value that fails.
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


// Expected stdout
// ---------------
// PASS ArithmeticProperties.absIsNonNegative
// PASS ArithmeticProperties.absIsIdempotent
// PASS ArithmeticProperties.nonZeroDividedByItselfIsOne
// FAIL ArithmeticProperties.valuesStayBelowTen
//   counterexample:
//     n = 10
//   AssertionError: expected true
//
// 3 passed, 1 failed
//
// Expected process exit status: 1
