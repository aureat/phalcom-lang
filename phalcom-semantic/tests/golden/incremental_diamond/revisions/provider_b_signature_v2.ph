// REVISION V4
// Provider B is restored with an incompatible parameter signature. Consumer's
// existing Int call must receive a targeted signature mismatch.

import app.base.Packet

class ProviderB {
  @class
  load(_ value: String) -> Packet<Int> {
    Packet<Int>.new(0)
  }
}

export ProviderB
