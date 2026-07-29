// Phase 04 — the standard registry resolves stable built-in reflective types
// and supports explicit user registration without runtime type enforcement.

import Assert from hypothesis
import Gen from hypothesis
import StrategyRegistry from hypothesis

const registry = StrategyRegistry.standard
Assert.equal("int", registry.forType(Int).fingerprint)
Assert.equal("bool", registry.forType(Bool).fingerprint)
Assert.equal("float", registry.forType(Float).fingerprint)
Assert.equal("bytes", registry.forType(Bytes).fingerprint)
Assert.equal("text", registry.forType(String).fingerprint)

class UserId {
  @constructor
  new() {}
}
registry.register(UserId, use: Gen.just(UserId.new()))
Assert.equal("just(UserId)", registry.forType(UserId).fingerprint)
Assert.isTrue({ registry.forType(List) }.attempt().isErr)

System.print("PASS strategy registry")
