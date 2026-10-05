// Phase 11: custom passes propose immutable candidates; Shrinker remains authoritative.

import { Assert, Choice, Example, ShrinkPass, Shrinker } from "hypothesis"

class DuplicateZeroPass {
  name -> Symbol { #duplicateZero }

  candidates(current: Example) -> List<Example> {
    const zero = current.replace(0, current.at(0).withValue(0))
    return const [zero, zero]
  }
}

const custom: ShrinkPass = DuplicateZeroPass.new()
const shrinker = Shrinker.new(passes: const [custom])
Assert.equal(#duplicateZero, custom.name)
Assert.equal(0, shrinker.acceptedComplexities.size)
