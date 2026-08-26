// REVISION V1
// Provider A body can change without changing its published callable signature.

import app.base.Packet

class ProviderA {
  @class
  load(_ value: Int) -> Packet<Int> {
    Packet<Int>.new(value)
  }
}

export ProviderA
