from universe.time.calendar.month import Month, Gregorian
from universe.time.calendar.period import Period

enum DateOverflow {
  Constrain
  Reject
}

data Date private(_ year: Int, _ month: Month, _ day: Int)

impl Date {
  @class
  new(_ year: Int, _ month: Month, _ day: Int) -> Date {
    if not (year is Int) { ArgumentError.new("Date.new: year must be an Int").raise() }
    if not (month is Month) { ArgumentError.new("Date.new: month must be a Month").raise() }
    if not (day is Int) { ArgumentError.new("Date.new: day must be an Int").raise() }
    const lastDay = Gregorian.daysInMonth(year, month.number)
    if day < 1 or day > lastDay {
      ArgumentError.new("Date.new: day is outside the month").raise()
    }
    Date(year, month, day)
  }

  daysInMonth -> Int { Gregorian.daysInMonth(self.year, self.month.number) }

  isLeapYear -> Bool { Gregorian.isLeapYear(self.year) }

  dayOfYear -> Int {
    Gregorian.daysFromCivil(self.year, 1, 1) |> (_ start) {
      Gregorian.daysFromCivil(self.year, self.month.number, self.day) - start + 1
    }
  }

  weekday -> Weekday {
    const epochDay = self.daysSince
    let number = (epochDay + 4) % 7
    if number < 0 { number = number + 7 }
    Weekday.fromNumber(number + 1)
  }

  daysSince -> Int { Gregorian.daysFromCivil(self.year, self.month.number, self.day) }

  compare(_ other: Date) -> Ordering {
    if not (other is Date) { ArgumentError.new("Date.compare: other must be a Date").raise() }
    self.daysSince.compare(other.daysSince)
  }

  <(_ other: Date) -> Bool { self.compare(other) === Ordering::Less }

  <=(_ other: Date) -> Bool {
    const order = self.compare(other)
    (order === Ordering::Less) or (order === Ordering::Equal)
  }

  >(_ other: Date) -> Bool { self.compare(other) === Ordering::Greater }

  >=(_ other: Date) -> Bool {
    const order = self.compare(other)
    (order === Ordering::Greater) or (order === Ordering::Equal)
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(Date) { return false }
    (self.year == other.year) and (self.month === other.month) and (self.day == other.day)
  }

  hash -> Int { (self.year * 1_000_000) + (self.month.number * 10_000) + self.day }

  advance(_ period: Period, overflow: DateOverflow) -> Date {
    if not (period is Period) { ArgumentError.new("Date.advance: period must be a Period").raise() }
    if not (overflow is DateOverflow) { ArgumentError.new("Date.advance: overflow must be a DateOverflow").raise() }

    const target = Gregorian.addMonths(self.year, self.month.number, period.months)
    const targetYear = target.year
    const targetMonth = Month.fromNumber(target.month)
    const targetLastDay = Gregorian.daysInMonth(targetYear, target.month)
    let targetDay = self.day
    if targetDay > targetLastDay {
      match overflow {
        Constrain => { targetDay = targetLastDay }
        Reject => { ArgumentError.new("Date.advance: target month cannot contain the original day").raise() }
      }
    }

    const monthAdjusted = Gregorian.daysFromCivil(targetYear, targetMonth.number, targetDay)
    const result = Gregorian.civilFromDays(monthAdjusted + period.days)
    Date.new(result.year, Month.fromNumber(result.month), result.day)
  }

  toString -> String {
    "#{self.year}-#{self.month.number}-#{self.day}"
  }
}

export Date, DateOverflow
