// Regression: nested span closure removes stack tails in place and freezes spans by id.

import { Assert, ScriptedChoiceProvider } from "hypothesis"
import DrawData from "choices/data"

const data = DrawData.new(
  provider: ScriptedChoiceProvider.new(const []),
  generationSize: 0,
  maxChoices: 1
)
let depth = 0
while depth < 1000 {
  data.withSpan(label: #nested, discardable: false) { None }
  depth++
}
Assert.equal(1000, data.example.spans.size)
