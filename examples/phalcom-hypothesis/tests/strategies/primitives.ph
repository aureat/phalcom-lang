// Phase 04 — primitive strategies consume typed DrawData choices and replay
// their exact values without consulting randomness.

import Assert from hypothesis
import Gen from hypothesis
import choice from "choices/choice"
import example from "choices/example"
import data from "choices/data"

const Choice = choice.Choice
const Example = example.Example
const DrawData = data.DrawData

const intExample = Example.from(
  choices: const [
    Choice.integer(value: -7, min: -7, max: 12, shrinkTowards: 0, label: Some.new(#integer))
  ],
  spans: const [],
  generationSize: 10
)
Assert.equal(
  -7,
  Gen.int(min: -7, max: 12).named(#integer).draw(
    DrawData.replay(example: intExample, maxChoices: 8)
  )
)

const defaultIntExample = Example.from(
  choices: const [
    Choice.integer(value: 16, min: -16, max: 16, shrinkTowards: 0, label: None)
  ],
  spans: const [],
  generationSize: 4
)
Assert.equal(
  16,
  Gen.int.draw(DrawData.replay(example: defaultIntExample, maxChoices: 8))
)

const boolExample = Example.from(
  choices: const [Choice.boolean(value: true, shrinkTowards: false, label: None)],
  spans: const [],
  generationSize: 1
)
Assert.equal(
  true,
  Gen.bool.draw(DrawData.replay(example: boolExample, maxChoices: 8))
)

const floatExample = Example.from(
  choices: const [
    Choice.integer(value: 1250000, min: -2000000, max: 2000000, shrinkTowards: 0, label: None)
  ],
  spans: const [],
  generationSize: 8
)
Assert.equal(
  1.25,
  Gen.float(min: -2.0, max: 2.0).draw(
    DrawData.replay(example: floatExample, maxChoices: 8)
  )
)

const payload = Bytes.zeroed(3)
payload[0] = 1
payload[1] = 2
payload[2] = 3
const bytesExample = Example.from(
  choices: const [
    Choice.bytes(
      value: payload,
      minSize: 1,
      maxSize: 4,
      shrinkTowards: Bytes.zeroed(1),
      label: None
    )
  ],
  spans: const [],
  generationSize: 4
)
Assert.equal(
  payload,
  Gen.bytes(minSize: 1, maxSize: 4).draw(
    DrawData.replay(example: bytesExample, maxChoices: 8)
  )
)

const textExample = Example.from(
  choices: const [
    Choice.integer(value: 2, min: 2, max: 2, shrinkTowards: 2, label: Some.new(#length)),
    Choice.index(value: 0, size: 1, shrinkTowards: 0, label: Some.new(#character)),
    Choice.index(value: 0, size: 1, shrinkTowards: 0, label: Some.new(#character))
  ],
  spans: const [],
  generationSize: 4
)
Assert.equal(
  "AA",
  Gen.text(
    alphabet: Gen.sampledFrom(const [65]),
    minSize: 2,
    maxSize: 2
  ).draw(DrawData.replay(example: textExample, maxChoices: 8))
)

System.print("PASS strategy primitives")
