// REVISION V1
// Provider B is the second branch in the consumer's dependency diamond.

import app.base.Packet

class ProviderB {
  @class
  load(_ value: Int) -> Packet<Int> {
    Packet<Int>.new(value + 1)
  }
}

export ProviderB
