// Phase 10: a sealed hierarchy derives oneOf over stable reflected variants.

import Assert from hypothesis
import StrategyRegistry from hypothesis
import arbitrary from hypothesis

@arbitrary
@data
@sealed
class Token {
  @variant Integer(value: Int)
  @variant Name(text: String)
}

const strategy = StrategyRegistry.standard.forType(Token)
Assert.true(strategy.fingerprint.includes("sealed(Token"))
Assert.true(strategy.fingerprint.includes("Integer"))
Assert.true(strategy.fingerprint.includes("Name"))
