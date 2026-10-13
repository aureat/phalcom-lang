from universe.functional import Chain
from universe.numbers import Rational
from experiments.probability import FiniteDistribution

class Dice {
  @class run() -> Unit {
    let faces = Chain<Int>::Empty.prepend(6).prepend(5).prepend(4).prepend(3).prepend(2).prepend(1)
    let die = FiniteDistribution<Int>.uniform(faces, equal: |a, b| { a == b }).unwrap
    let sum = die.zipWith(die, combine: |a, b| { a + b }, equal: |a, b| { a == b })
    System.print(sum.mass(7))
    System.print(sum.expectation(|value| { Rational.fromInt(value) }))
    System.print(sum.variance(|value| { Rational.fromInt(value) }))
    System.print(sum.condition(|value| { value >= 10 }))
    System.print(die.condition(|value| { value > 6 }))
    ()
  }
}
export Dice
