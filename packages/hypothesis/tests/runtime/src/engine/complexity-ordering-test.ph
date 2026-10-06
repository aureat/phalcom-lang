from hypothesis.choices.choice import Choice
import hypothesis.choices.example as Examples
import hypothesis.engine.complexity as Complexities
import hypothesis_tests.support.test as Testing

let mut oneChoice = List.new()
oneChoice.append(Choice::Integer(50, 0, 100, 0))
let mut twoMinimalChoices = List.new()
twoMinimalChoices.append(Choice::Integer(0, 0, 100, 0))
twoMinimalChoices.append(Choice::Integer(0, 0, 100, 0))

let smallerTape = Complexities.ExampleComplexity.of(
  example: Examples.Example.new(choices: oneChoice)
)
let largerTape = Complexities.ExampleComplexity.of(
  example: Examples.Example.new(choices: twoMinimalChoices)
)
Testing.Test.expect(
  smallerTape.lessThan(other: largerTape),
  "choice count must precede magnitude in complexity ordering"
)

let mut shortValues = List.new()
shortValues.append(Choice::Integer(0, 0, 100, 0))
shortValues.append(Choice::Integer(5, 0, 100, 0))
let mut longValues = List.new()
longValues.append(Choice::Integer(0, 0, 100, 0))
longValues.append(Choice::Integer(20, 0, 100, 0))

let shorterComplexity = Complexities.ExampleComplexity.of(
  example: Examples.Example.new(choices: shortValues)
)
let longerComplexity = Complexities.ExampleComplexity.of(
  example: Examples.Example.new(choices: longValues)
)
Testing.Test.expect(
  shorterComplexity.lessThan(other: longerComplexity),
  "magnitude must break ties in choice count"
)
Testing.Test.pass("engine.complexity_ordering")
