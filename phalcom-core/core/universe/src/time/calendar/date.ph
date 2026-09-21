from universe.time.calendar.month import Month, Gregorian
from universe.time.calendar.period import Period
from universe.time.calendar.weekday import Weekday
from universe.time.timezone import TimeZone
from universe.time.timestamp import Timestamp

enum DateOverflow {
  Constrain
  Reject
}

data Date private(_ year: Int, _ month: Month, _ day: Int)

impl Date {
  @class
  new(_ year: Int, _ month: Month, _ day: Int) -> Date {
    const lastDay = Gregorian.daysInMonth(year, month.number)
    if day < 1 or day > lastDay {
      ArgumentError.new("Date.new: day is outside the month").raise()
    }
    Date(year, month, day)
  }

  @class
  parse(_ text: String) -> Date {
    const parts = text.split("-")
    if parts.size != 3 { ArgumentError.new("Date.parse: expected YYYY-MM-DD").raise() }
    Date.new(Date.parseInteger(parts.at(0)), Month.fromNumber(Date.parseInteger(parts.at(1))), Date.parseInteger(parts.at(2)))
  }

  @class
  private parseInteger(_ text: String) -> Int {
    if text.size == 0 { ArgumentError.new("Date.parse: invalid integer").raise() }
    let sign = 1
    let index = 0
    if text.codePointAt(0) == 45 { sign = -1; index = 1 }
    let value = 0
    while index < text.size {
      const cp = text.codePointAt(index)
      if cp < 48 or cp > 57 { ArgumentError.new("Date.parse: invalid digit").raise() }
      value = value * 10 + cp - 48
      index = index + 1
    }
    value * sign
  }

  @class
  today -> Date { Date.fromLocalParts(TimeZone.system.localNative(Timestamp.now.unixNanoseconds)) }

  @class
  todayIn(_ zone: TimeZone) -> Date { Date.fromLocalParts(zone.localNative(Timestamp.now.unixNanoseconds)) }

  @class
  private fromLocalParts(_ parts: List<Int>) -> Date {
    Date.new(parts.at(0), Month.fromNumber(parts.at(1)), parts.at(2))
  }

  daysInMonth -> Int { Gregorian.daysInMonth(self.year, self.month.number) }

  isLeapYear -> Bool { Gregorian.isLeapYear(self.year) }

  dayOfYear -> Int {
    const start = Gregorian.daysFromCivil(self.year, 1, 1)
    Gregorian.daysFromCivil(self.year, self.month.number, self.day) - start + 1
  }

  weekday -> Weekday {
    const epochDay = self.daysSince
    let number = (epochDay + 4) % 7
    if number < 0 { number = number + 7 }
    Weekday.fromNumber(number + 1)
  }

  daysSince -> Int { Gregorian.daysFromCivil(self.year, self.month.number, self.day) }

  compare(_ other: Date) -> Ordering {
    self.daysSince.compare(other.daysSince)
  }

  <(_ other: Date) -> Bool { self.compare(other) === Ordering::Less }

  <=(_ other: Date) -> Bool {
    const order = self.compare(other)
    if order === Ordering::Less { true } else { order === Ordering::Equal }
  }

  >(_ other: Date) -> Bool { self.compare(other) === Ordering::Greater }

  >=(_ other: Date) -> Bool {
    const order = self.compare(other)
    if order === Ordering::Greater { true } else { order === Ordering::Equal }
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(Date) { return false }
    (self.year == other.year) and (self.month === other.month) and (self.day == other.day)
  }

  hash -> Int { (self.year * 1_000_000) + (self.month.number * 10_000) + self.day }

  advance(_ period: Period, onOverflow overflowStrategy: DateOverflow) -> Date {
    if period is not Period {
        ArgumentError.new("Date.advance: period must be a Period").raise()
    }

    if overflowStrategy is not DateOverflow {
        ArgumentError.new("Date.advance: overflow must be a DateOverflow").raise()
    }

    const targetYear = Gregorian.addMonthsYear(self.year, self.month.number, period.months)
    const targetMonthNumber = Gregorian.addMonthsNumber(self.year, self.month.number, period.months)
    const targetMonth = Month.fromNumber(targetMonthNumber)
    const targetLastDay = Gregorian.daysInMonth(targetYear, targetMonthNumber)

    let targetDay = self.day
    if targetDay > targetLastDay {
      if overflowStrategy === DateOverflow::Constrain {
        targetDay = targetLastDay
      } else {
        ArgumentError.new("Date.advance: target month cannot contain the original day").raise()
      }
    }

    const monthAdjusted = Gregorian.daysFromCivil(targetYear, targetMonth.number, targetDay)
    const result = Gregorian.civilFromDays(monthAdjusted + period.days)
    Date.new(
      result.get(#year).unwrapOr(-1),
      Month.fromNumber(result.get(#month).unwrapOr(-1)),
      result.get(#day).unwrapOr(-1)
    )
  }

  toString -> String {
    const yearText = "\(self.year)"
    const monthText = if self.month.number < 10 { "0\(self.month.number)" } else { "\(self.month.number)" }
    const dayText = if self.day < 10 { "0\(self.day)" } else { "\(self.day)" }
    "\(yearText)-\(monthText)-\(dayText)"
  }
}

export Date, DateOverflow
