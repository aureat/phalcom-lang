// Phase 10: an installed @strategy(Type) provider overrides automatic derivation.

import Assert from hypothesis
import Gen from hypothesis
import Strategy from hypothesis
import StrategyRegistry from hypothesis
import arbitrary from hypothesis
import strategy from hypothesis

@arbitrary
@data
@immutable
class UserId {
  const _value: Int
}

class DomainStrategies {
  @strategy(UserId)
  userIds() -> Strategy<UserId> {
    return Gen.just(UserId.new(value: 42))
  }
}

const registry = StrategyRegistry.standard.register(DomainStrategies)
Assert.equal("just(UserId)", registry.forType(UserId).fingerprint)
