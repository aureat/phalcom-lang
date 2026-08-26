// REVISION V5
// Compatible Provider B publication is restored; prior missing/mismatch state
// must not poison the reusable branch or shared Packet identity.

import app.base.Packet

class ProviderB {
  @class
  load(_ value: Int) -> Packet<Int> {
    Packet<Int>.new(value + 1)
  }
}

export ProviderB
