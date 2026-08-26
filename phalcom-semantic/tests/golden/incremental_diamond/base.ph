// REVISION BASE
// Shared generic packet identity for both provider branches.

class Packet<T> {
  _value: T

  @constructor
  new(_ value: T) {
    _value = value
  }

  value() -> T { _value }
}

class IntPacket is Packet<Int> {}

export Packet, IntPacket
