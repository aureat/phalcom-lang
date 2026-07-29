// Phase 03 — a mutable buffer freezes by value. Later buffer or returned-list
// mutation cannot change the immutable semantic example.

import Assert from hypothesis
import choice from "choices/choice"
import buffer from "choices/buffer"

const Choice = choice.Choice
const ChoiceBuffer = buffer.ChoiceBuffer

const choices = ChoiceBuffer.new(generationSize: 12)
choices.add(
  Choice.integer(value: 3, min: 0, max: 10, shrinkTowards: 0, label: None)
)

const frozen = choices.freeze
choices.add(
  Choice.boolean(value: true, shrinkTowards: false, label: None)
)

Assert.equal(1, frozen.size)
Assert.equal(2, choices.size)
Assert.equal(3, frozen.at(0).value)
Assert.equal(12, frozen.generationSize)

const exposed = frozen.choices
exposed.add(
  Choice.index(value: 0, size: 1, shrinkTowards: 0, label: None)
)
Assert.equal(1, frozen.size)

const sourceBytes = Bytes.zeroed(2)
sourceBytes[0] = 7
sourceBytes[1] = 8
const byteBuffer = ChoiceBuffer.new(generationSize: 4)
byteBuffer.add(
  Choice.bytes(
    value: sourceBytes,
    minSize: 0,
    maxSize: 4,
    shrinkTowards: Bytes.zeroed(0),
    label: None
  )
)
const frozenBytes = byteBuffer.freeze
sourceBytes[0] = 99
Assert.equal(7, frozenBytes.at(0).value[0])

const returnedBytes = frozenBytes.at(0).value
returnedBytes[0] = 42
Assert.equal(7, frozenBytes.at(0).value[0])

System.print("PASS choices buffer freeze")
