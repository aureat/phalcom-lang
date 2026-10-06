from hypothesis.choices.choice import Choice
import hypothesis_tests.support.test as Testing

let integer = Choice::Integer(7, 0, 10, 0)
let boolean = Choice::Boolean(true, false)
let index = Choice::Index(2, 5, 0)
let bytes = Bytes.zeroed(2)
bytes[0] = 10
bytes[1] = 20
let byteChoice = Choice::Bytes(bytes, 0, 4, Bytes.zeroed(0))

match integer {
  Choice::Integer(value, min, max, target) => {
    Testing.Test.expect(
      value == 7 and min == 0 and max == 10 and target == 0,
      "integer choice fields changed"
    )
  }
}

match boolean {
  Choice::Boolean(value, target) => {
    Testing.Test.expect(value and not target, "boolean choice fields changed")
  }
}

match index {
  Choice::Index(value, size, target) => {
    Testing.Test.expect(value == 2 and size == 5 and target == 0, "index choice fields changed")
  }
}

match byteChoice {
  Choice::Bytes(value, minSize, maxSize, _) => {
    Testing.Test.expect(
      value.size == 2 and value[0] == 10 and value[1] == 20,
      "byte choice payload changed"
    )
    Testing.Test.expect(
      minSize == 0 and maxSize == 4,
      "byte choice bounds changed"
    )
  }
}

Testing.Test.pass("choices.choice_variants")
