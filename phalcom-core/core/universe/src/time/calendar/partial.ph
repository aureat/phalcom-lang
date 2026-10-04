from universe.time.calendar.date import Date
from universe.time.calendar.month import Month, Gregorian

data YearMonth private(_ year: Int, _ month: Month)

impl YearMonth {
  @class
  new(_ year: Int, _ month: Month) -> YearMonth {
    YearMonth(year, month)
  }

  days -> Int { Gregorian.daysInMonth(self.year, self.month.number) }

  isLeapYear -> Bool { Gregorian.isLeapYear(self.year) }

  plusMonths(_ amount: Int) -> YearMonth {
    let resultYear = Gregorian.addMonthsYear(self.year, self.month.number, amount)
    let resultMonth = Gregorian.addMonthsNumber(self.year, self.month.number, amount)
    YearMonth.new(resultYear, Month.fromNumber(resultMonth))
  }

  minusMonths(_ amount: Int) -> YearMonth { self.plusMonths(-amount) }

  atDay(_ day: Int) -> Date { Date.new(self.year, self.month, day) }
}

data MonthDay private(_ month: Month, _ day: Int)

impl MonthDay {
  @class
  new(_ month: Month, _ day: Int) -> MonthDay {
    let maximum = Gregorian.daysInMonth(2000, month.number)
    if day < 1 or day > maximum {
      ArgumentError.new("MonthDay.new: day is outside the month").raise()
    }
    MonthDay(month, day)
  }

  inYear(_ year: Int) -> Date {
    Date.new(year, self.month, self.day)
  }
}

export YearMonth, MonthDay
