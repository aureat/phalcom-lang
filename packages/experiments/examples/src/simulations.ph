from universe.random import Xoshiro256PlusPlus
from experiments.sampling import Sample

class Simulations {
  @class run() -> Unit {
    let die = Sample<Xoshiro256PlusPlus, Int>.integer(from: 1, below: 7)
    let twoDice = die.zipWith(die, combine: |left, right| { left + right })
    let generator = Xoshiro256PlusPlus.seed(2026)
    System.print(twoDice.replicate(8).run(using: generator))
    // The second die's bound depends on the first die's outcome.
    let dependent = die.flatMap(|sides| { Sample<Xoshiro256PlusPlus, Int>.integer(from: 1, below: sides + 1) })
    System.print(dependent.run(using: generator))
    System.print(twoDice.filter(|sum| { sum >= 10 }, attempts: 20).run(using: generator))
    ()
  }
}
export Simulations
