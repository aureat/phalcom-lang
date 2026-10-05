import .choices.provider as Providers
import .choices.buffer as Buffers
import .choices.data as ChoiceData
import .strategies.primitives as PrimitiveStrategies
from .choices.choice import Choice

// Adapted from the legacy primitive replay test: a scripted integer choice
// must pass through the active strategy and draw-data APIs unchanged.
let mut script = List.new()
script.add(Choice::Integer(7, 0, 100, 0))

let provider = Providers.ScriptedChoiceProvider.new(script: script)
let drawData = ChoiceData.DrawData.new(
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

System.print("PASS Hypothesis integer strategy smoke")
