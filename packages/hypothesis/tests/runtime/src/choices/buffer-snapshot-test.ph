import hypothesis.choices.buffer as Buffers
from hypothesis.choices.choice import Choice
import hypothesis_tests.support.test as Testing

let mut buffer = Buffers.ChoiceBuffer.new()
buffer.append(Choice::Integer(3, 0, 10, 0))
let example = buffer.snapshot
buffer.append(Choice::Boolean(true, false))

Testing.Test.expect(buffer.size == 2, "buffer must retain choices appended after snapshot")
Testing.Test.expect(example.size == 1, "snapshot must be independent of later buffer writes")
match example.at(index: 0) {
  Choice::Integer(value, _, _, _) =>
    Testing.Test.expect(value == 3, "snapshot must preserve the original choice")
  _ => { throw Error.new("expected integer choice in buffer snapshot") }
}

Testing.Test.pass("choices.buffer_snapshot")
