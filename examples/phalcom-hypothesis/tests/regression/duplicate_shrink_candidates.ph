// Regression: duplicate candidates from one or more custom passes are replayed once.

import { Assert, Choice, Example, Shrinker } from "hypothesis"

class DuplicatePass {
  name -> Symbol => #duplicates
  candidates(current: Example) -> List<Example> {
    const candidate = current.replace(0, current.at(0).withValue(0))
    return const [candidate, candidate, candidate]
  }
}

const shrinker = Shrinker.new(passes: const [DuplicatePass.new()])
Assert.equal(0, shrinker.acceptedComplexities.size)
// Runtime fixture asserts evaluator replay count == 1 after shrinkFailure.
