// Phase 05 — recursive shrinking collapses an expanded subtree while retaining
// the enclosing structural decision and later sibling decision.

import Assert from hypothesis
import Settings from hypothesis
import Phase from hypothesis
import Gen from hypothesis
import choice from "choices/choice"
import example from "choices/example"
import specification from "engine/specification"
import engine from "engine/engine"

const Choice = choice.Choice
const Example = example.Example
const tree = Gen.recursive(
  base: Gen.just(#leaf),
  extend: { child => Gen.build { draw =>
    draw.from(Gen.int(min: 0, max: 9))
    return draw.from(Gen.tuple(child, child))
  } }
)
const seedExample = Example.from(
  choices: const [
    Choice.boolean(value: true, shrinkTowards: false, label: Some.new(#recursive)),
    Choice.integer(value: 5, min: 0, max: 9, shrinkTowards: 0, label: None),
    Choice.boolean(value: true, shrinkTowards: false, label: Some.new(#recursive)),
    Choice.integer(value: 7, min: 0, max: 9, shrinkTowards: 0, label: None),
    Choice.boolean(value: false, shrinkTowards: false, label: Some.new(#recursive))
  ],
  spans: const [],
  generationSize: 2
)
const spec = specification.PropertySpec.check(
  id: #recursiveSubtree,
  target: { value => Assert.isFalse(value.isA(Tuple)) },
  strategies: const [tree],
  explicitExamples: const [],
  reuseExamples: const [seedExample],
  settings: Settings.standard.phases(const [Phase.Reuse, Phase.Shrink])
)

const result = engine.SearchEngine.new().check(spec)
Assert.equal(
  Tuple.fromList(const [#leaf, #leaf]),
  result.args.at(0)
)
Assert.equal(4, result.tape.unwrap.size)

System.print("PASS engine recursive subtree deletion")
