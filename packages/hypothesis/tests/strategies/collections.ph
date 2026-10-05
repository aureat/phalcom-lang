// Phase 04 — standard collection strategies are deterministic and list
// elements receive discardable semantic spans.

import Assert from hypothesis
import Gen from hypothesis
import choice from "choices/choice"
import example from "choices/example"
import data from "choices/data"

const Choice = choice.Choice
const Example = example.Example
const DrawData = data.DrawData

const listExample = Example.from(
  choices: const [
    Choice.integer(value: 2, min: 0, max: 3, shrinkTowards: 0, label: Some.new(#length)),
    Choice.integer(value: 7, min: 0, max: 9, shrinkTowards: 0, label: None),
    Choice.integer(value: 8, min: 0, max: 9, shrinkTowards: 0, label: None)
  ],
  spans: const [],
  generationSize: 4
)
const listData = DrawData.replay(example: listExample, maxChoices: 16)
Assert.equal(
  const [7, 8],
  Gen.list(of: Gen.int(min: 0, max: 9), minSize: 0, maxSize: 3).draw(listData)
)
Assert.equal(3, listData.example.spans.size)
Assert.equal(#list, listData.example.spans.at(0).label)
Assert.equal(#element, listData.example.spans.at(1).label)
Assert.isTrue(listData.example.spans.at(1).discardable)
Assert.equal(#element, listData.example.spans.at(2).label)
Assert.isTrue(listData.example.spans.at(2).discardable)

const tupleExample = Example.from(
  choices: const [
    Choice.boolean(value: true, shrinkTowards: false, label: None),
    Choice.integer(value: 3, min: 3, max: 3, shrinkTowards: 3, label: None)
  ],
  spans: const [],
  generationSize: 2
)
Assert.equal(
  Tuple.__fromList(const [true, 3]),
  Gen.tuple(Gen.bool, Gen.int(min: 3, max: 3)).draw(
    DrawData.replay(example: tupleExample, maxChoices: 8)
  )
)

Assert.equal("option(int)", Gen.option(Gen.int).fingerprint)
Assert.equal(
  "result(int,text)",
  Gen.result(ok: Gen.int, error: Gen.text).fingerprint
)
Assert.equal(
  "set(int,0,4)",
  Gen.set(of: Gen.int, minSize: 0, maxSize: 4).fingerprint
)
Assert.equal(
  "map(text,int,0,4)",
  Gen.map(
    keys: Gen.text,
    values: Gen.int,
    minSize: 0,
    maxSize: 4
  ).fingerprint
)

System.print("PASS strategy collections")
