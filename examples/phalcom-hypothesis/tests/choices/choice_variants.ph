// Phase 03 — primitive choices and requests retain typed semantic metadata.

import Assert from hypothesis
import choice from "choices/choice"
import request from "choices/request"

const Choice = choice.Choice
const ChoiceRequest = request.ChoiceRequest

const bytes = Bytes.zeroed(3)
bytes[0] = 1
bytes[1] = 2
bytes[2] = 3

const choices = const [
  Choice.integer(value: 7, min: -10, max: 10, shrinkTowards: 0, label: Some.new(#number)),
  Choice.boolean(value: true, shrinkTowards: false, label: Some.new(#flag)),
  Choice.index(value: 2, size: 5, shrinkTowards: 0, label: Some.new(#branch)),
  Choice.bytes(value: bytes, minSize: 0, maxSize: 8, shrinkTowards: Bytes.zeroed(0), label: Some.new(#payload))
]

const labels = List.new()
for item in choices {
  labels.add(
    item.match(
      integer: { value => value.label.unwrap },
      boolean: { value => value.label.unwrap },
      index: { value => value.label.unwrap },
      bytes: { value => value.label.unwrap }
    )
  )
}
Assert.equal(const [#number, #flag, #branch, #payload], labels)
Assert.equal(7, choices.at(0).value)
Assert.equal(true, choices.at(1).value)
Assert.equal(2, choices.at(2).value)
Assert.equal(bytes, choices.at(3).value)

const requests = const [
  ChoiceRequest.integer(min: -10, max: 10, shrinkTowards: 0, label: Some.new(#number)),
  ChoiceRequest.boolean(shrinkTowards: false, label: Some.new(#flag)),
  ChoiceRequest.index(size: 5, shrinkTowards: 0, label: Some.new(#branch)),
  ChoiceRequest.bytes(minSize: 0, maxSize: 8, shrinkTowards: Bytes.zeroed(0), label: Some.new(#payload))
]
Assert.equal(0, requests.at(0).shrinkTarget)
Assert.equal(false, requests.at(1).shrinkTarget)
Assert.equal(0, requests.at(2).shrinkTarget)
Assert.equal(Bytes.zeroed(0), requests.at(3).shrinkTarget)

Assert.isTrue({
  ChoiceRequest.integer(min: 4, max: 3, shrinkTowards: 4, label: None)
}.attempt().isErr)
Assert.isTrue({
  ChoiceRequest.integer(min: 0, max: 3, shrinkTowards: 4, label: None)
}.attempt().isErr)
Assert.isTrue({
  ChoiceRequest.index(size: 0, shrinkTowards: 0, label: None)
}.attempt().isErr)
Assert.isTrue({
  ChoiceRequest.bytes(minSize: 4, maxSize: 3, shrinkTowards: Bytes.zeroed(4), label: None)
}.attempt().isErr)

System.print("PASS choices variants")
