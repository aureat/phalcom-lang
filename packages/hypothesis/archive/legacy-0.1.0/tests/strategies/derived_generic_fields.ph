// Phase 10: constructor parameters may contain recursively resolved applied types.

import Assert from hypothesis
import StrategyRegistry from hypothesis
import arbitrary from hypothesis

@arbitrary
@data
@immutable
class Payload {
  const _number: Option<Int>
  const _names: List<String>
}

const fingerprint = StrategyRegistry.standard.forType(Payload).fingerprint
Assert.true(fingerprint.includes("option(int)"))
Assert.true(fingerprint.includes("list(text"))
