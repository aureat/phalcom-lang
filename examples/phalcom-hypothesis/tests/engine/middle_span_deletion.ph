// Phase 05 — shrinking can delete a middle collection element while retaining
// a later relevant element; prefix truncation alone cannot produce [1, 2].

import Assert from hypothesis
import Settings from hypothesis
import Phase from hypothesis
import Gen from hypothesis
import choice from "choices/choice"
import span from "choices/span"
import example from "choices/example"
import specification from "engine/specification"
import engine from "engine/engine"

const Choice = choice.Choice
const Span = span.Span
const Example = example.Example

const seedExample = Example.from(
  choices: const [
    Choice.integer(value: 3, min: 0, max: 5, shrinkTowards: 0, label: Some.new(#length)),
    Choice.integer(value: 1, min: 0, max: 100, shrinkTowards: 0, label: None),
    Choice.integer(value: 99, min: 0, max: 100, shrinkTowards: 0, label: None),
    Choice.integer(value: 2, min: 0, max: 100, shrinkTowards: 0, label: None)
  ],
  spans: const [
    Span.create(id: 0, label: #list, start: 0, end: 4, parent: None, discardable: false),
    Span.create(id: 1, label: #element, start: 1, end: 2, parent: Some.new(0), discardable: true),
    Span.create(id: 2, label: #element, start: 2, end: 3, parent: Some.new(0), discardable: true),
    Span.create(id: 3, label: #element, start: 3, end: 4, parent: Some.new(0), discardable: true)
  ],
  generationSize: 5
)
const spec = specification.PropertySpec.check(
  id: #middleDeletion,
  target: { values =>
    Assert.isFalse(
      values.size >= 2 and values.at(0) == 1 and values.at(values.size - 1) == 2
    )
  },
  strategies: const [Gen.list(of: Gen.int(min: 0, max: 100), minSize: 0, maxSize: 5)],
  explicitExamples: const [],
  reuseExamples: const [seedExample],
  settings: Settings.standard.phases(const [Phase.Reuse, Phase.Shrink])
)

const result = engine.SearchEngine.new().check(spec)
Assert.equal(const [1, 2], result.args.at(0))
Assert.equal(3, result.tape.unwrap.size)

System.print("PASS engine middle span deletion")
