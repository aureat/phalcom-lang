// Phase 04 — every standard strategy replays its normalized semantic example
// without consulting randomness.

import Assert from hypothesis
import Gen from hypothesis
import data from "choices/data"

const DrawData = data.DrawData

const strategies = const [
  Gen.int,
  Gen.bool,
  Gen.float,
  Gen.bytes,
  Gen.text,
  Gen.just(17),
  Gen.sampledFrom(const [#a, #b, #c]),
  Gen.oneOf(Gen.int(min: 0, max: 4), Gen.just(9)),
  Gen.option(Gen.int(min: -2, max: 2)),
  Gen.result(ok: Gen.int(min: 0, max: 4), error: Gen.text),
  Gen.list(of: Gen.int(min: 0, max: 4), minSize: 0, maxSize: 4),
  Gen.set(of: Gen.int(min: 0, max: 20), minSize: 0, maxSize: 4),
  Gen.map(
    keys: Gen.sampledFrom(const [#a, #b, #c, #d]),
    values: Gen.int(min: 0, max: 9),
    minSize: 0,
    maxSize: 3
  ),
  Gen.tuple(Gen.bool, Gen.int(min: 0, max: 4)),
  Gen.build |draw| {
    const value = draw.from(Gen.int(min: 0, max: 4))
    return Tuple.__fromList(const [value, value])
  },
  Gen.deferred || { Gen.just(#deferred) },
  Gen.recursive(
    base: Gen.just(0),
    extend: |child| { Gen.tuple(child, child) }
  )
]

let index = 0
for strategy in strategies {
  const generated = DrawData.generate(
    random: Random.new(seed: 20260723 + index),
    generationSize: 4,
    maxChoices: 256
  )
  const generatedValue = strategy.draw(generated)
  const generatedExample = generated.example

  const replayed = DrawData.replay(
    example: generatedExample,
    maxChoices: 256
  )
  const replayedValue = strategy.draw(replayed)

  Assert.equal(generatedValue, replayedValue)
  Assert.equal(generatedExample, replayed.example)
  index++
}

System.print("PASS deterministic strategy replay")
