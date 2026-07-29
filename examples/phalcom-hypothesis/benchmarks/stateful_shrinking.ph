// Phase 11 benchmark workload: structural deletion across many stateful actions.
import { Assert, StateMachine, Stateful, Rule, Settings } from "hypothesis"

class CounterMachine is StateMachine {
  @constructor
  new() { _value = 0 }

  @Rule
  increment() { _value++ }

  @Rule
  failAtLimit() { Assert.true(_value < 50) }
}

class StatefulShrinkingBenchmark {
  @class
  run(steps: Int) -> Any {
    return Stateful.check(
      CounterMachine,
      with: Settings.standard.statefulSteps(steps).maxExamples(1)
    )
  }
}
