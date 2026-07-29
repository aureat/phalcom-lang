// Phase 05 — the accepted minimal example deterministically replays to the
// same arguments and stable failure origin.

import Assert from hypothesis
import Settings from hypothesis
import Phase from hypothesis
import Gen from hypothesis
import choice from "choices/choice"
import example from "choices/example"
import specification from "engine/specification"
import evaluator from "engine/evaluator"
import engine from "engine/engine"

const seedExample = example.Example.from(
  choices: const [
    choice.Choice.integer(value: 77, min: 0, max: 100, shrinkTowards: 0, label: None)
  ],
  spans: const [],
  generationSize: 7
)
const spec = specification.PropertySpec.check(
  id: #deterministicMinimalReplay,
  target: { n => Assert.isTrue(n < 10) },
  strategies: const [Gen.int(min: 0, max: 100)],
  explicitExamples: const [],
  reuseExamples: const [seedExample],
  settings: Settings.standard.phases(const [Phase.Reuse, Phase.Shrink])
)
const result = engine.SearchEngine.new().check(spec)
const worker = evaluator._Evaluator.new(spec)
const first = worker.replay(result.tape.unwrap).status
const second = worker.replay(result.tape.unwrap).status
Assert.equal(first.args, second.args)
Assert.isTrue(first.match(
  valid: { _ => false },
  invalid: { _ => false },
  overrun: { _ => false },
  interesting: { value => second.match(
    valid: { _ => false },
    invalid: { _ => false },
    overrun: { _ => false },
    interesting: { other => value.failure.sameOrigin(other.failure) }
  ) }
))

System.print("PASS engine deterministic minimal replay")
