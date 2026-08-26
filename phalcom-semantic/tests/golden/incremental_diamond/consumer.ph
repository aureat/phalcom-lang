// REVISION V1
// Consumer depends on both provider branches and the shared Packet identity.

import app.base.Packet
import app.provider_a.ProviderA
import app.provider_b.ProviderB

class Consumer {
  @class
  choose(_ value: Int, _ useA: Bool) -> Packet<Int> {
    if useA {
      ProviderA.load(value)
    } else {
      ProviderB.load(value)
    }
  }
}

export Consumer
