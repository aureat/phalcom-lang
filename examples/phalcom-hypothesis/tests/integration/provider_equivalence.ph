// Phase 11: scripted and seeded-system providers execute the same property
// when they supply the same normalized primitive choices.

import {
  Assert,
  Choice,
  Gen,
  ScriptedChoiceProvider,
  SystemRandomChoiceProvider
} from "hypothesis"
import DrawData from "choices/data"

const strategy = Gen.tuple(Gen.int(min: 0, max: 10), Gen.bool)
const systemData = DrawData.new(
  provider: SystemRandomChoiceProvider.new(Random.new(seed: 23)),
  generationSize: 5,
  maxChoices: 10
)
const systemValue = strategy.draw(systemData)
const scriptedData = DrawData.new(
  provider: ScriptedChoiceProvider.new(systemData.example.choices),
  generationSize: 5,
  maxChoices: 10
)
const scriptedValue = strategy.draw(scriptedData)
Assert.equal(systemValue, scriptedValue)
Assert.equal(systemData.example, scriptedData.example)
