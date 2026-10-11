import universe.math as math
from universe.random import Bernoulli
from .support import FiniteSupport

class BernoulliSupport {
  @class
  of(_ distribution: Bernoulli) -> FiniteSupport<Bool> {
    let values = List<Bool>.new()
    if distribution.probability < 1.0 { values.append(false) }
    if distribution.probability > 0.0 { values.append(true) }
    return FiniteSupport.from(values)
  }
}

/// Returns the positive-mass Boolean support of a random-owned Bernoulli law.
let bernoulliSupport = &BernoulliSupport.of(_)

export Bernoulli, bernoulliSupport
