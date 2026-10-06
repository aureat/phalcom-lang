// BLOCKED: [probe=conformance callable has no declaration owner] the compiler currently panics while lowering the generic Strategy.draw conformance.
from hypothesis.choices.provider import ScriptedChoiceProvider
from hypothesis.choices.buffer import ChoiceBuffer
from hypothesis.choices.data import DrawData
from hypothesis.strategies.primitives import IntStrategy
from hypothesis.choices.choice import Choice

let mut script = List.new()
script.append(Choice::Integer(7, 0, 100, 0))

let provider = ScriptedChoiceProvider.new(script: script)
let drawData = DrawData<ScriptedChoiceProvider>.new(
  provider: provider,
  buffer: ChoiceBuffer.new()
)
let strategy = IntStrategy(
  min: 0,
  max: 100,
  shrinkTowards: 0
)
let value = strategy.draw(drawData: drawData)

if value != 7 or drawData.consumedChoices != 1 {
  throw Error.new("scripted integer strategy did not replay its choice")
}

System.print("PASS strategies.integer_replay")
