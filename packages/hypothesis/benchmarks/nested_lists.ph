// Phase 11 benchmark workload: nested collection generation and span freezing.
import { Gen, SystemRandomChoiceProvider } from "hypothesis"
import DrawData from "choices/data"

class NestedListsBenchmark {
  @class
  run(iterations: Int) -> Int {
    const strategy = Gen.list(of: Gen.list(of: Gen.int, minSize: 8, maxSize: 8), minSize: 8, maxSize: 8)
    let total = 0
    let index = 0
    while index < iterations {
      const data = DrawData.new(
        provider: SystemRandomChoiceProvider.new(Random.new(seed: index + 1)),
        generationSize: 100,
        maxChoices: 1000
      )
      strategy.draw(data)
      total += data.example.spans.size
      index++
    }
    return total
  }
}
