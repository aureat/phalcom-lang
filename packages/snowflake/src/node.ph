from universe.time.timestamp import Timestamp
from .errors import SnowflakeGenerationError
from .layout import SnowflakeLayout
from .id import SnowflakeId

/// Stateful, node-local generator for sequential callers. Node IDs must be
/// allocated uniquely across machines; this class does not coordinate nodes.
class SnowflakeNode {
  _nodeId: Int
  _layout: SnowflakeLayout
  mut _started: Bool
  mut _lastMillis: Int
  mut _sequence: Int

  @constructor
  private create(_ nodeId: Int, _ layout: SnowflakeLayout) {
    self._nodeId = nodeId
    self._layout = layout
    self._started = false
    self._lastMillis = 0
    self._sequence = 0
  }

  @class
  new(_ nodeId: Int) -> SnowflakeNode {
    SnowflakeNode.configured(nodeId: nodeId, layout: SnowflakeLayout.twitter)
  }

  @class
  configured(nodeId: Int, layout: SnowflakeLayout) -> SnowflakeNode {
    if nodeId < 0 or nodeId > layout.nodeMax {
      throw ArgumentError.new("Snowflake node ID is outside the configured node-bit range")
    }
    SnowflakeNode.create(nodeId, layout)
  }

  nodeId -> Int { self._nodeId }
  layout -> SnowflakeLayout { self._layout }

  /// Raises rather than wrapping on clock rollback, sequence exhaustion, or
  /// timestamp overflow. Repeated calls in one millisecond advance the step.
  generate() -> SnowflakeId {
    self.generateAt(Timestamp.now.unixNanoseconds ~/ 1000000)
  }

  /// A deterministic time input for testing and externally managed clocks.
  /// Do not share one instance across concurrent callers without coordination.
  generateAt(_ unixMillis: Int) -> SnowflakeId {
    let delta = unixMillis - self._layout.epochMillis
    if delta < 0 or delta > self._layout.maxDeltaMillis {
      throw SnowflakeGenerationError.new("Snowflake timestamp is outside its layout range")
    }
    if self._started and unixMillis < self._lastMillis {
      throw SnowflakeGenerationError.new("Snowflake clock moved backward")
    }
    if self._started and unixMillis == self._lastMillis and self._sequence == self._layout.sequenceMax {
      throw SnowflakeGenerationError.new("Snowflake sequence exhausted for this millisecond")
    }
    let next = if self._started and unixMillis == self._lastMillis { self._sequence + 1 } else { 0 }
    let bits = (delta << self._layout.timeShift) | (self._nodeId << self._layout.sequenceBits) | next
    let id = SnowflakeId.assembled(bits)
    self._started = true
    self._lastMillis = unixMillis
    self._sequence = next
    id
  }
}

export SnowflakeNode
