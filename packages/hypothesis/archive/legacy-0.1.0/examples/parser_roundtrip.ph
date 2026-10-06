// Phase 05 structural-shrinking example. A deliberately faulty token codec
// fails whenever a sequence begins with #open and ends with #close. The seed
// contains irrelevant middle noise, so the minimal counterexample requires
// deleting that middle element while retaining the later closing token.

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
const seed = Example.from(
  choices: const [
    Choice.integer(value: 3, min: 0, max: 8, shrinkTowards: 0, label: Some.new(#length)),
    Choice.index(value: 0, size: 3, shrinkTowards: 0, label: None),
    Choice.index(value: 1, size: 3, shrinkTowards: 0, label: None),
    Choice.index(value: 2, size: 3, shrinkTowards: 0, label: None)
  ],
  spans: const [
    Span.create(id: 0, label: #list, start: 0, end: 4, parent: None, discardable: false),
    Span.create(id: 1, label: #element, start: 1, end: 2, parent: Some.new(0), discardable: true),
    Span.create(id: 2, label: #element, start: 2, end: 3, parent: Some.new(0), discardable: true),
    Span.create(id: 3, label: #element, start: 3, end: 4, parent: Some.new(0), discardable: true)
  ],
  generationSize: 4
)
const tokens = Gen.list(
  of: Gen.sampledFrom(const [#open, #noise, #close]),
  minSize: 0,
  maxSize: 8
)
const spec = specification.PropertySpec.check(
  id: #parserRoundTrip,
  target: |values| {
    const broken = values.size >= 2 and
      values.at(0) == #open and
      values.at(values.size - 1) == #close
    Assert.isFalse(broken)
  },
  strategies: const [tokens],
  explicitExamples: const [],
  reuseExamples: const [seed],
  settings: Settings.standard.phases(const [Phase.Reuse, Phase.Shrink])
)

const result = engine.SearchEngine.new().check(spec)
System.print("minimal token sequence: " + result.args.at(0).toString)
