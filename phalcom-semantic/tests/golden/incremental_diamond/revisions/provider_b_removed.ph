// REVISION V3
// Provider B's public export is removed. Consumer's import edge must become a
// structured missing-publication result rather than stale success.

import app.base.Packet

class PrivateProvider {
  @class
  load(_ value: Int) -> Packet<Int> {
    Packet<Int>.new(value)
  }
}
