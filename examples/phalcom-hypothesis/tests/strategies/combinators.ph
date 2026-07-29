// Phase 04 — strategy combinators compose over the same DrawData stream.

import Assert from hypothesis
import Gen from hypothesis
import choice from "choices/choice"
import example from "choices/example"
import data from "choices/data"

const Choice = choice.Choice
const Example = example.Example
const DrawData = data.DrawData

const composedExample = Example.from(
  choices: const [
    Choice.integer(value: -1, min: -2, max: 2, shrinkTowards: 0, label: None),
    Choice.integer(value: 2, min: -2, max: 2, shrinkTowards: 0, label: None),
    Choice.integer(value: 4, min: 4, max: 4, shrinkTowards: 4, label: None)
  ],
  spans: const [],
  generationSize: 4
)
const composedData = DrawData.replay(example: composedExample, maxChoices: 16)
const value = Gen.int(min: -2, max: 2)
  .filter { candidate => candidate > 0 }
  .map { candidate => candidate * 2 }
  .flatMap { candidate => Gen.int(min: candidate, max: candidate) }
  .draw(composedData)

Assert.equal(4, value)
Assert.equal(1, composedData.rejectionCount)

const namedExample = Example.from(
  choices: const [
    Choice.integer(value: 1, min: 0, max: 1, shrinkTowards: 0, label: None)
  ],
  spans: const [],
  generationSize: 1
)
const namedData = DrawData.replay(example: namedExample, maxChoices: 8)
Assert.equal(1, Gen.int(min: 0, max: 1).named(#namedValue).draw(namedData))
Assert.equal(#namedValue, namedData.example.spans.at(0).label)

System.print("PASS strategy combinators")
