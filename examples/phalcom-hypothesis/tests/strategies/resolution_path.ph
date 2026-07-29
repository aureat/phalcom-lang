// Phase 10: unsupported nested fields report the complete resolution path.

import Assert from hypothesis
import StrategyRegistry from hypothesis
import StrategyResolutionError from hypothesis
import arbitrary from hypothesis

class Opaque {}

@arbitrary
@data
@immutable
class Envelope {
  const _payload: List<Opaque>
}

const outcome = { StrategyRegistry.standard.forType(Envelope) }.attempt()
outcome.match(
  ok: { _ => Assert.fail("expected derivation to fail") },
  error: { error =>
    Assert.true(error.isA(StrategyResolutionError))
    Assert.true(error.message.unwrap.includes("resolution path"))
    Assert.true(error.message.unwrap.includes("Envelope"))
    Assert.true(error.message.unwrap.includes("payload"))
    Assert.true(error.message.unwrap.includes("List<Opaque>"))
    Assert.true(error.message.unwrap.includes("Opaque"))
  }
)
