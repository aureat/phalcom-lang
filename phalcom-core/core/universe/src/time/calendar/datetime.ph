from universe.time.calendar.date import Date, DateOverflow
from universe.time.calendar.period import Period
from universe.time.calendar.time import Time
from universe.time.calendar.month import Month
from universe.time.offset import UtcOffset
from universe.time.timezone import TimeZone
from universe.time.timestamp import Timestamp
from universe.time.duration import Duration

data DateTime private(_ date: Date, _ time: Time)

impl DateTime {
  @class
  new(_ date: Date, _ time: Time) -> DateTime {
    DateTime(date, time)
  }

  @class
  parse(_ text: String) -> DateTime {
    let separator = text.indexOf("T")
    if separator <= 0 { ArgumentError.new("DateTime.parse: expected <date>T<time>").raise() }
    DateTime.new(Date.parse(text.slice(0, separator).toString), Time.parse(text.slice(separator + 1, text.size).toString))
  }

  compare(_ other: DateTime) -> Ordering {
    let dateOrder = self.date.compare(other.date)
    if dateOrder != Ordering::Equal { return dateOrder }
    self.time.compare(other.time)
  }

  <(_ other: DateTime) -> Bool { self.compare(other) === Ordering::Less }

  <=(_ other: DateTime) -> Bool {
    let order = self.compare(other)
    if order === Ordering::Less { true } else { order === Ordering::Equal }
  }

  >(_ other: DateTime) -> Bool { self.compare(other) === Ordering::Greater }

  >=(_ other: DateTime) -> Bool {
    let order = self.compare(other)
    if order === Ordering::Greater { true } else { order === Ordering::Equal }
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(DateTime) { return false }
    if self.date == other.date { self.time == other.time } else { false }
  }

  hash -> Int { (self.date.hash * 1_000_000_000) + self.time.hash }

  toString -> String { "\(self.date)T\(self.time)" }

  advance(_ period: Period, overflow: DateOverflow) -> DateTime {
    DateTime.new(self.date.advance(period, overflow), self.time)
  }

  private timestampFromResolution(_ encoded: List<Int>, _ index: Int) -> Timestamp {
    Timestamp.seconds(encoded.at(index)) + Duration.nanoseconds(encoded.at(index + 1))
  }

  resolve(_ zone: TimeZone) -> LocalTimeResolution {
    let encoded = zone.resolveNative(self.date.year, self.date.month.number, self.date.day, self.time.hour, self.time.minute, self.time.second, self.time.nanosecond)
    let tag = encoded.at(0)
    if tag == 0 { return LocalTimeResolution::Unique(value: ZonedDateTime.from(self.timestampFromResolution(encoded, 1), zone)) }
    if tag == 1 {
      return LocalTimeResolution::Ambiguous(
        earlier: ZonedDateTime.from(self.timestampFromResolution(encoded, 1), zone),
        later: ZonedDateTime.from(self.timestampFromResolution(encoded, 3), zone)
      )
    }
    LocalTimeResolution::Gap(
      before: ZonedDateTime.from(self.timestampFromResolution(encoded, 1), zone),
      after: ZonedDateTime.from(self.timestampFromResolution(encoded, 3), zone)
    )
  }

  inZone(_ zone: TimeZone) -> ZonedDateTime { self.inZone(zone, LocalTimeDisambiguation::Reject) }

  inZone(_ zone: TimeZone, _ disambiguation: LocalTimeDisambiguation) -> ZonedDateTime {
    self.resolve(zone).resolve(disambiguation)
  }

}

data LocalTimeDisambiguation { Reject Earlier Later Compatible }

data ZonedDateTime private(_ timestamp: Timestamp, _ timeZone: TimeZone)

data LocalTimeResolution {
  Unique(value: ZonedDateTime)
  Ambiguous(earlier: ZonedDateTime, later: ZonedDateTime)
  Gap(before: ZonedDateTime, after: ZonedDateTime)
}

impl LocalTimeResolution {
  resolve(_ disambiguation: LocalTimeDisambiguation) -> ZonedDateTime {
    match self {
      Unique(value) => value
      Ambiguous(earlier: earlier, later: later) => {
        if disambiguation === LocalTimeDisambiguation::Reject { ArgumentError.new("local time is ambiguous").raise() }
        if disambiguation === LocalTimeDisambiguation::Later { later } else { earlier }
      }
      Gap(before: before, after: after) => {
        if disambiguation === LocalTimeDisambiguation::Reject { ArgumentError.new("local time is in a gap").raise() }
        if disambiguation === LocalTimeDisambiguation::Earlier { before } else { after }
      }
    }
  }

}

impl ZonedDateTime {
  @class
  from(_ timestamp: Timestamp, _ timeZone: TimeZone) -> ZonedDateTime { ZonedDateTime(timestamp, timeZone) }
  @class
  now -> ZonedDateTime { ZonedDateTime.from(Timestamp.now, TimeZone.system) }
  @class
  nowIn(_ zone: TimeZone) -> ZonedDateTime { ZonedDateTime.from(Timestamp.now, zone) }
  offset -> UtcOffset { UtcOffset.seconds(self.timeZone.offsetSeconds(self.timestamp)) }
  internal localParts -> List<Int> { self.timeZone.localNative(self.timestamp.unixNanoseconds) }
  date -> Date { Date.new(self.localParts.at(0), Month.fromNumber(self.localParts.at(1)), self.localParts.at(2)) }
  time -> Time { Time.new(self.localParts.at(3), self.localParts.at(4), self.localParts.at(5), self.localParts.at(6)) }
  withTimeZone(_ zone: TimeZone) -> ZonedDateTime { ZonedDateTime(self.timestamp, zone) }
  shift(_ duration: Duration) -> ZonedDateTime { ZonedDateTime(self.timestamp + duration, self.timeZone) }
  dateTime -> DateTime { DateTime.new(self.date, self.time) }
  advance(_ period: Period) -> ZonedDateTime { self.dateTime.advance(period, DateOverflow::Constrain).inZone(self.timeZone, LocalTimeDisambiguation::Compatible) }
  sameInstant(_ other: ZonedDateTime) -> Bool { self.timestamp == other.timestamp }
  compareInstant(_ other: ZonedDateTime) -> Ordering { self.timestamp.compare(other.timestamp) }
  ==(_ other: Dynamic) -> Bool {
    if not other.is(ZonedDateTime) {
      return false
    }
    if self.timestamp == other.timestamp {
      self.timeZone == other.timeZone
    } else {
      false
    }
  }

  toString -> String { "\(self.date)T\(self.time)\(self.offset)[\(self.timeZone)]" }
}
export DateTime, ZonedDateTime, LocalTimeDisambiguation, LocalTimeResolution
