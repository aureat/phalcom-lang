// Size-aware automatic derivation for a recursive sealed expression tree.

import Assert from hypothesis
import Given from hypothesis
import PropertyRunner from hypothesis
import PropertySuite from hypothesis
import Settings from hypothesis
import arbitrary from hypothesis

@arbitrary
@data
@sealed
class Expression {
  @variant Literal(value: Int)
  @variant Negate(value: Expression)
  @variant Add(_ left: Expression, _ right: Expression)
}

class ExpressionProperties is PropertySuite {
  @Given
  printingIsDeterministic(expression: Expression) {
    Assert.equal(expression.toString, expression.toString)
  }
}

PropertyRunner.run(
  const [ExpressionProperties],
  with: Settings.standard.examples(100)
)
