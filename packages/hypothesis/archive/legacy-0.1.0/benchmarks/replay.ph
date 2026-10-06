// Phase 11 benchmark workload: replay normalization over a retained example.
import { Gen, SystemRandomChoiceProvider } from "hypothesis"
import DrawData from "choices/data"

class ReplayBenchmark {
  @class
  run(iterations: Int) -> Int {
    const strategy = Gen.list(of: Gen.int, minSize: 64, maxSize: 64)
    const generated = DrawData.new(
      provider: SystemRandomChoiceProvider.new(Random.new(seed: 9)),
      generationSize: 100,
      maxChoices: 1000
    )
    strategy.draw(generated)
    let index = 0
    let choices = 0
    while index < iterations {
      const replay = DrawData.replay(example: generated.example, maxChoices: 1000)
      strategy.draw(replay)
      choices += replay.consumedChoices
      index++
    }
    return choices
  }
}
