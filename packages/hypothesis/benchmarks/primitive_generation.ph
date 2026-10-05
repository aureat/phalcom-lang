// Phase 11 benchmark workload: primitive generation throughput.
import { Gen, SystemRandomChoiceProvider } from "hypothesis"
import DrawData from "choices/data"

class PrimitiveGenerationBenchmark {
  @class
  run(iterations: Int) -> Int {
    const provider = SystemRandomChoiceProvider.new(Random.new(seed: 1))
    const data = DrawData.new(provider: provider, generationSize: 50, maxChoices: iterations * 4)
    let index = 0
    while index < iterations {
      Gen.int.draw(data)
      Gen.bool.draw(data)
      index++
    }
    return data.consumedChoices
  }
}
