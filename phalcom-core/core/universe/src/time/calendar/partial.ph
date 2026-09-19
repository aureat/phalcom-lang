from universe.time.calendar.date import Date
from universe.time.calendar.month import Month, Gregorian

data YearMonth private(_ year: Int, _ month: Month)

impl YearMonth {
  @class
  new(_ year: Int, _ month: Month) -> YearMonth {
    if not (year is Int) { ArgumentError.new("YearMonth.new: year must be an Int").raise() }
    if not (month is Month) { ArgumentError.new("YearMonth.new: month must be a Month").raise() }
    YearMonth(year, month)
  }

  days -> Int { Gregorian.daysInMonth(self.year, self.month.number) }

  isLeapYear -> Bool { Gregorian.isLeapYear(self.year) }

  plusMonths(_ amount: Int) -> YearMonth {
    if not (amount is Int) { ArgumentError.new("YearMonth.plusMonths: amount must be an Int").raise() }
    const resultYear = Gregorian.addMonthsYear(self.year, self.month.number, amount)
    const resultMonth = Gregorian.addMonthsNumber(self.year, self.month.number, amount)
    YearMonth.new(resultYear, Month.fromNumber(resultMonth))
  }

  minusMonths(_ amount: Int) -> YearMonth { self.plusMonths(-amount) }

  atDay(_ day: Int) -> Date { Date.new(self.year, self.month, day) }
}

data MonthDay private(_ month: Month, _ day: Int)

impl MonthDay {
  @class
  new(_ month: Month, _ day: Int) -> MonthDay {
    if not (month is Month) { ArgumentError.new("MonthDay.new: month must be a Month").raise() }
    if not (day is Int) { ArgumentError.new("MonthDay.new: day must be an Int").raise() }
    const maximum = Gregorian.daysInMonth(2000, month.number)
    if day < 1 or day > maximum {
      ArgumentError.new("MonthDay.new: day is outside the month").raise()
    }
    MonthDay(month, day)
  }

  inYear(_ year: Int) -> Date {
    if not (year is Int) { ArgumentError.new("MonthDay.inYear: year must be an Int").raise() }
    Date.new(year, self.month, self.day)
  }
}

export YearMonth, MonthDay
