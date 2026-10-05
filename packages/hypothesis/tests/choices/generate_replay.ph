// Phase 03 — generation records typed primitive choices and replay supplies
// the same values without consulting randomness. Replayed choices normalize to
// the current request metadata.

import Assert from hypothesis
import data from "choices/data"

const generated = data.DrawData.generate(
  random: Random.new(seed: 20260723),
  generationSize: 18,
  maxChoices: 20
)

const generatedValues = generated.withSpan(label: #sample, discardable: false) {
  return const [
    generated.drawInt(min: -20, max: 20, shrinkTowards: 0, label: Some.new(#number)),
    generated.drawBool(shrinkTowards: false, label: Some.new(#flag)),
    generated.drawIndex(size: 7, shrinkTowards: 0, label: Some.new(#branch)),
    generated.drawBytes(minSize: 2, maxSize: 5, shrinkTowards: Bytes.zeroed(2), label: Some.new(#payload))
  ]
}
const generatedExample = generated.example

const replayed = data.DrawData.replay(
  example: generatedExample,
  maxChoices: 20
)
const replayedValues = replayed.withSpan(label: #sample, discardable: false) {
  return const [
    replayed.drawInt(min: -20, max: 20, shrinkTowards: 0, label: Some.new(#number)),
    replayed.drawBool(shrinkTowards: false, label: Some.new(#flag)),
    replayed.drawIndex(size: 7, shrinkTowards: 0, label: Some.new(#branch)),
    replayed.drawBytes(minSize: 2, maxSize: 5, shrinkTowards: Bytes.zeroed(2), label: Some.new(#payload))
  ]
}
const replayedExample = replayed.example

Assert.equal(generatedValues, replayedValues)
Assert.equal(generatedExample, replayedExample)
Assert.equal(generatedExample.signature, replayedExample.signature)
Assert.equal(4, replayed.consumedChoices)

System.print("PASS choices generate replay")
