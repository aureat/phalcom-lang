from hypothesis.choices.choice import Choice
import hypothesis.choices.example as Examples
import hypothesis.engine.complexity as Complexities
import hypothesis_tests.support.test as Testing

let mut oneChoice = List.new()
oneChoice.add(Choice::Integer(50, 0, 100, 0))
let mut twoMinimalChoices = List.new()
twoMinimalChoices.add(Choice::Integer(0, 0, 100, 0))
twoMinimalChoices.add(Choice::Integer(0, 0, 100, 0))

let smallerTape = Complexities.ExampleComplexity.of(Examples.Example.new(choices: oneChoice))
let largerTape = Complexities.ExampleComplexity.of(Examples.Example.new(choices: twoMinimalChoices))
Testing.Test.expect(
  smallerTape.lessThan(largerTape),
  "choice count must precede magnitude in complexity ordering"
)

let mut shortValues = List.new()
shortValues.add(Choice::Integer(0, 0, 100, 0))
shortValues.add(Choice::Integer(5, 0, 100, 0))
let mut longValues = List.new()
longValues.add(Choice::Integer(0, 0, 100, 0))
longValues.add(Choice::Integer(20, 0, 100, 0))

let shorterComplexity = Complexities.ExampleComplexity.of(Examples.Example.new(choices: shortValues))
let longerComplexity = Complexities.ExampleComplexity.of(Examples.Example.new(choices: longValues))
Testing.Test.expect(
  shorterComplexity.lessThan(longerComplexity),
  "magnitude must break ties in choice count"
)
Testing.Test.pass("engine.complexity_ordering")
