// Phase 06: bare @Given resolves primitive and recursive applied annotations.

import Assert from hypothesis
import Given from hypothesis
import PropertySuite from hypothesis
import PropertyRunner from hypothesis
import Settings from hypothesis

class InferredProperties is PropertySuite {
  @Given
  primitiveValues(
    integer: Int,
    flag: Bool,
    text: String,
    bytes: Bytes
  ) {
    self.assertTrue(integer.isA(Int))
    self.assertTrue(flag.isA(Bool))
    self.assertTrue(text.isA(String))
    self.assertTrue(bytes.isA(Bytes))
  }

  @Given
  recursiveValues(
    optional: Option<Int>,
    values: List<Bool>,
    pair: Tuple<Int, String>
  ) {
    self.assertTrue(optional.isA(Option))
    self.assertTrue(values.isA(List))
    self.assertTrue(pair.isA(Tuple))
  }
}

const run = PropertyRunner.run(
  const [InferredProperties],
  with: Settings.standard.examples(5)
)
Assert.equal(2, run.passedCount)
