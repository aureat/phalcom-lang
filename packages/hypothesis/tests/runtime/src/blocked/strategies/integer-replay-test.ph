// BLOCKED: [probe=DrawData,ChoiceProvider] awaiting runtime re-probe after generic refactor.
import hypothesis.choices.provider as Providers
import hypothesis.choices.buffer as Buffers
import hypothesis.choices.data as ChoiceData
import hypothesis.strategies.primitives as PrimitiveStrategies
from hypothesis.choices.choice import Choice

let mut script = List.new()
script.add(Choice::Integer(7, 0, 100, 0))

let provider = Providers.ScriptedChoiceProvider.new(script: script)
let drawData = ChoiceData.DrawData<Providers.ScriptedChoiceProvider>.new(
  provider: provider,
  buffer: Buffers.ChoiceBuffer.new()
)
let strategy = PrimitiveStrategies.IntStrategy(
  min: 0,
  max: 100,
  shrinkTowards: 0
)
let value = strategy.draw(drawData)

if value != 7 or drawData.consumedChoices != 1 {
  throw Error.new("scripted integer strategy did not replay its choice")
}

System.print("PASS strategies.integer_replay")
