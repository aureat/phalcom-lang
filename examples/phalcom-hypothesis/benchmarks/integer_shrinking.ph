// Phase 11 benchmark workload: ordered integer candidate generation.
import { Choice, Example, Shrinker } from "hypothesis"

class IntegerShrinkingBenchmark {
  @class
  seed(size: Int) -> Example {
    const choices = List.new()
    let index = 0
    while index < size {
      choices.add(Choice.integer(value: 1000, min: -1000, max: 1000, shrinkTowards: 0, label: None))
      index++
    }
    return Example.from(choices: choices, spans: const [], generationSize: 100)
  }
}
