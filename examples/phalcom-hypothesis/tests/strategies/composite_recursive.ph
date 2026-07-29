// Phase 04 — explicit draws, deferred factories, and recursive strategies all
// use the same DrawData and recursive expansion terminates at size zero.

import Assert from hypothesis
import Gen from hypothesis
import choice from "choices/choice"
import example from "choices/example"
import data from "choices/data"

const Choice = choice.Choice
const Example = example.Example
const DrawData = data.DrawData

const builtExample = Example.from(
  choices: const [
    Choice.integer(value: 4, min: 0, max: 10, shrinkTowards: 0, label: None),
    Choice.integer(value: 4, min: 4, max: 4, shrinkTowards: 4, label: None)
  ],
  spans: const [],
  generationSize: 4
)
const pair = Gen.build { draw =>
  const first = draw.from(Gen.int(min: 0, max: 10))
  const second = draw.from(Gen.int(min: first, max: first))
  return Tuple.fromList(const [first, second])
}.draw(DrawData.replay(example: builtExample, maxChoices: 8))
Assert.equal(Tuple.fromList(const [4, 4]), pair)

const deferred = Gen.deferred { Gen.just(42) }
Assert.equal(42, deferred.draw(DrawData.generate(
  random: Random.new(seed: 1),
  generationSize: 0,
  maxChoices: 8
)))

const recursive = Gen.recursive(
  base: Gen.just(0),
  extend: { child => Gen.tuple(child, child) }
)
Assert.equal(
  0,
  recursive.draw(DrawData.generate(
    random: Random.new(seed: 1),
    generationSize: 0,
    maxChoices: 8
  ))
)

System.print("PASS composite and recursive strategies")
