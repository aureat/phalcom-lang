from universe.time.timestamp import Timestamp
from universe.time.offset import UtcOffset

data ZoneTransition private(
  _ timestamp: Timestamp,
  _ offsetBefore: UtcOffset,
  _ offsetAfter: UtcOffset
)

impl ZoneTransition {
  @class
  new(_ timestamp: Timestamp, _ offsetBefore: UtcOffset, _ offsetAfter: UtcOffset) -> ZoneTransition {
    ZoneTransition(timestamp, offsetBefore, offsetAfter)
  }

  toString -> String { "\(self.timestamp)\(self.offsetBefore)->\(self.offsetAfter)" }
}

export ZoneTransition
