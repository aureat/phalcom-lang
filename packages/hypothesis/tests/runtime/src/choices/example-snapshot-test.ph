from hypothesis.choices.choice import Choice
import hypothesis.choices.example as Examples
import hypothesis_tests.support.test as Testing

let sourceBytes = Bytes.new(2)
sourceBytes[0] = 10
sourceBytes[1] = 20
let mut sourceChoices = List.new()
sourceChoices.append(Choice::Bytes(sourceBytes, 0, 2, Bytes.new(0)))

let example = Examples.Example.new(choices: sourceChoices)
sourceBytes[0] = 99
sourceChoices.append(Choice::Boolean(true, false))

Testing.Test.expect(example.size == 1, "Example must snapshot its source list")
match example.at(index: 0) {
  Choice::Bytes(value, _, _, _) => {
    Testing.Test.expect(
      value[0] == 10 and value[1] == 20,
      "Example must snapshot mutable byte payloads"
    )
    value[0] = 77
  }
  _ => { throw Error.new("expected byte choice in example") }
}

let mut exposedChoices = example.choices
exposedChoices.append(Choice::Boolean(false, false))
Testing.Test.expect(example.size == 1, "Example.choices must return an isolated list")

match example.at(index: 0) {
  Choice::Bytes(value, _, _, _) =>
    Testing.Test.expect(value[0] == 10, "Example.at must return an isolated byte payload")
  _ => { throw Error.new("expected byte choice in example") }
}

Testing.Test.pass("choices.example_snapshot")
