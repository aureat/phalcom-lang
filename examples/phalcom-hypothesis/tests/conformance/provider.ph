// Phase 11: public providers obey one normalization and exhaustion contract.

import {
  Assert,
  Choice,
  ChoiceProvider,
  ScriptedChoiceProvider,
  SystemRandomChoiceProvider
} from "hypothesis"
import ChoiceRequest from "choices/request"

const request = ChoiceRequest.integer(
  min: -5,
  max: 5,
  shrinkTowards: 0,
  label: Some.new(#normalized)
)
const scripted = ScriptedChoiceProvider.new(
  const [
    Choice.integer(
      value: 3,
      min: -100,
      max: 100,
      shrinkTowards: -100,
      label: None
    )
  ]
)
const scriptedChoice = scripted.choose(request)
Assert.equal(3, scriptedChoice.value)
Assert.equal(-5, scriptedChoice.min)
Assert.equal(5, scriptedChoice.max)
Assert.equal(0, scriptedChoice.shrinkTowards)
Assert.equal(Some.new(#normalized), scriptedChoice.label)
Assert.equal(1, scripted.consumedChoices)

const random = SystemRandomChoiceProvider.new(Random.new(seed: 11))
const randomChoice = random.choose(request)
Assert.true(randomChoice.value >= -5 and randomChoice.value <= 5)
Assert.equal(Some.new(#normalized), randomChoice.label)
