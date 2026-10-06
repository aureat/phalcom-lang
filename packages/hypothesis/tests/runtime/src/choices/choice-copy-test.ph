from hypothesis.choices.choice import Choice
import hypothesis_tests.support.test as Testing

let source = Bytes.new(2)
source[0] = 10
source[1] = 20
let shrinkTarget = Bytes.new(1)
shrinkTarget[0] = 3

let copied = Choice::Bytes(source, 0, 2, shrinkTarget).copy
source[0] = 99
shrinkTarget[0] = 88

match copied {
  Choice::Bytes(value, _, _, target) => {
    Testing.Test.expect(
      value[0] == 10 and value[1] == 20,
      "Choice.copy must copy byte payloads"
    )
    Testing.Test.expect(
      target[0] == 3,
      "Choice.copy must copy byte shrink targets"
    )
    value[0] = 44
    Testing.Test.expect(value[0] == 44, "copied byte payload must remain mutable by its owner")
  }
  _ => { throw Error.new("expected copied byte choice") }
}

Testing.Test.expect(source[0] == 99, "mutating a copied choice must not change its source")
Testing.Test.pass("choices.choice_copy")
