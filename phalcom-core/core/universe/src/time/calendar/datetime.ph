from universe.time.calendar.date import Date, DateOverflow
from universe.time.calendar.period import Period
from universe.time.calendar.time import Time

data DateTime private(_ date: Date, _ time: Time)

impl DateTime {
  @class
  new(_ date: Date, _ time: Time) -> DateTime {
    DateTime(date, time)
  }

  compare(_ other: DateTime) -> Ordering {
    const dateOrder = self.date.compare(other.date)
    if dateOrder != Ordering::Equal { return dateOrder }
    self.time.compare(other.time)
  }

  <(_ other: DateTime) -> Bool { self.compare(other) === Ordering::Less }

  <=(_ other: DateTime) -> Bool {
    const order = self.compare(other)
    if order === Ordering::Less { true } else { order === Ordering::Equal }
  }

  >(_ other: DateTime) -> Bool { self.compare(other) === Ordering::Greater }

  >=(_ other: DateTime) -> Bool {
    const order = self.compare(other)
    if order === Ordering::Greater { true } else { order === Ordering::Equal }
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(DateTime) { return false }
    if self.date == other.date { self.time == other.time } else { false }
  }

  hash -> Int { (self.date.hash * 1_000_000_000) + self.time.hash }

  advance(_ period: Period, overflow: DateOverflow) -> DateTime {
    DateTime.new(self.date.advance(period, overflow), self.time)
  }
}

export DateTime
