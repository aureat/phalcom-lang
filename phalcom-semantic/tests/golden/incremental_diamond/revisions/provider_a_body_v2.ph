// REVISION V2
// Same module surface as provider_a.ph; body edit should invalidate A and
// downstream consumers while leaving B's branch facts reusable.

import app.base.Packet

class ProviderA {
  @class
  load(_ value: Int) -> Packet<Int> {
    Packet<Int>.new(value + 10)
  }
}

export ProviderA
