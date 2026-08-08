// Phase 03 — nested semantic spans preserve source order, parent identity,
// choice ranges, and discardability.

import Assert from hypothesis
import choice from "choices/choice"
import buffer from "choices/buffer"

const Choice = choice.Choice
const ChoiceBuffer = buffer.ChoiceBuffer
const choices = ChoiceBuffer.new(generationSize: 20)

choices.withSpan(label: #composite, discardable: false) {
  choices.add(
    Choice.integer(value: 2, min: 0, max: 5, shrinkTowards: 0, label: Some.new(#length))
  )

  choices.withSpan(label: #element, discardable: true) {
    choices.add(
      Choice.integer(value: 10, min: -100, max: 100, shrinkTowards: 0, label: None)
    )
  }

  choices.withSpan(label: #element, discardable: true) {
    choices.add(
      Choice.integer(value: 20, min: -100, max: 100, shrinkTowards: 0, label: None)
    )
  }
}

const example = choices.freeze
Assert.equal(3, example.spans.size)

Assert.equal(const [0, 1, 2], example.spans.map |span| { span.id })

const root = example.spanWithId(0).unwrap
const first = example.spanWithId(1).unwrap
const second = example.spanWithId(2).unwrap

Assert.equal(#composite, root.label)
Assert.equal(0, root.start)
Assert.equal(3, root.end)
Assert.equal(None, root.parent)
Assert.isFalse(root.discardable)

Assert.equal(Some.new(0), first.parent)
Assert.equal(1, first.start)
Assert.equal(2, first.end)
Assert.isTrue(first.discardable)

Assert.equal(Some.new(0), second.parent)
Assert.equal(2, second.start)
Assert.equal(3, second.end)
Assert.isTrue(second.discardable)

System.print("PASS choices span tree")
