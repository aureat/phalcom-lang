/// An immutable interpretation of a Snowflake's timestamp, node, and sequence.
/// Each generator captures one layout; no process-wide mutable bit settings exist.
data SnowflakeLayout private(_ epochMillis: Int, _ nodeBits: Int, _ sequenceBits: Int)

impl SnowflakeLayout {
  @class
  twitter -> SnowflakeLayout { SnowflakeLayout(1288834974657, 10, 12) }

  @class
  configured(epochMillis: Int, nodeBits: Int, sequenceBits: Int) -> SnowflakeLayout {
    if nodeBits < 0 or sequenceBits < 0 or nodeBits + sequenceBits > 22 {
      throw ArgumentError.new("Snowflake layout requires 0..22 node/sequence bits totaling at most 22")
    }
    SnowflakeLayout(epochMillis, nodeBits, sequenceBits)
  }

  epochMillis -> Int { self._epochMillis }
  nodeBits -> Int { self._nodeBits }
  sequenceBits -> Int { self._sequenceBits }
  nodeMax -> Int { (1 << self._nodeBits) - 1 }
  sequenceMax -> Int { (1 << self._sequenceBits) - 1 }
  timeShift -> Int { self._nodeBits + self._sequenceBits }
  maxDeltaMillis -> Int { (1 << (63 - self.timeShift)) - 1 }
}

export SnowflakeLayout
