// Phase 03 — replay exhaustion and choice-budget exhaustion are engine
// overruns. They are never classified as interesting counterexamples.

import Assert from hypothesis
import data from "choices/data"
import example from "choices/example"

const exhausted = data.DrawData.replay(
  example: example.Example.empty,
  maxChoices: 10
).attempt |draw| {
  draw.drawInt(min: 0, max: 1, shrinkTowards: 0, label: None)
}

const limited = data.DrawData.generate(
  random: Random.new(seed: 1),
  generationSize: 0,
  maxChoices: 1
).attempt |draw| {
  draw.drawInt(min: 0, max: 1, shrinkTowards: 0, label: None)
  draw.drawInt(min: 0, max: 1, shrinkTowards: 0, label: None)
}

for status in const [exhausted, limited] {
  Assert.isTrue(status.overrun)
  Assert.isFalse(status.failed)
  Assert.equal(
    #overrun,
    status.match(
      valid: |_| { #valid },
      invalid: |_| { #invalid },
      overrun: |_| { #overrun },
      interesting: |_| { #interesting }
    )
  )
}

System.print("PASS choices overrun")
