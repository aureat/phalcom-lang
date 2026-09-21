from universe.time.offset import UtcOffset
from universe.time.duration import Duration
from universe.time.timestamp import Timestamp
from universe.time.zone_transition import ZoneTransition

class TimeZone {
  @class
  utc -> TimeZone {
    TimeZone.utcNative
  }

  @class
  fixed(_ offset: UtcOffset) -> TimeZone {
    TimeZone.fixedNative(offset.seconds)
  }

  @class
  named(_ identifier: String) -> TimeZone {
    TimeZone.namedNative(identifier)
  }

  @class
  system -> TimeZone {
    TimeZone.systemNative
  }

  @class
  internal native utcNative -> TimeZone

  @class
  internal native fixedNative(_ seconds: Int) -> TimeZone

  @class
  internal native namedNative(_ identifier: String) -> TimeZone

  @class
  internal native systemNative -> TimeZone

  offsetSeconds(_ timestamp: Timestamp) -> Int {
    self.offsetSecondsNative(timestamp.unixNanoseconds)
  }

  internal native offsetSecondsNative(_ timestampNanoseconds: Int) -> Int

  internal native identityNative -> Int
  internal native displayNative -> String

  ==(_ other: Dynamic) -> Bool {
    if not other.is(TimeZone) { return false }
    self.identityNative == other.identityNative
  }

  hash -> Int { self.identityNative }

  toString -> String { self.displayNative }

  internal native localNative(_ timestampNanoseconds: Int) -> List<Int>

  internal native resolveNative(
    _ year: Int,
    _ month: Int,
    _ day: Int,
    _ hour: Int,
    _ minute: Int,
    _ second: Int,
    _ nanosecond: Int
  ) -> List<Int>

  transitionBefore(_ timestamp: Timestamp) -> Option<ZoneTransition> {
    self.transitionValue(self.transitionBeforeNative(timestamp.unixNanoseconds))
  }

  transitionAfter(_ timestamp: Timestamp) -> Option<ZoneTransition> {
    self.transitionValue(self.transitionAfterNative(timestamp.unixNanoseconds))
  }

  private transitionValue(_ encoded: List<Int>) -> Option<ZoneTransition> {
    if encoded.size == 0 { return None }
    Some(ZoneTransition.new(
      Timestamp.seconds(encoded.at(0)) + Duration.nanoseconds(encoded.at(1)),
      UtcOffset.seconds(encoded.at(2)),
      UtcOffset.seconds(encoded.at(3))
    ))
  }

  internal native transitionBeforeNative(_ timestampNanoseconds: Int) -> List<Int>
  internal native transitionAfterNative(_ timestampNanoseconds: Int) -> List<Int>
}

export TimeZone
