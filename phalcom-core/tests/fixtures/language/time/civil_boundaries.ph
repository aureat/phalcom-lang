// area: time
// spec: stdlib/time.md
// status: PASS

from universe.time.calendar.date import Date, DateOverflow
from universe.time.calendar.month import Month
from universe.time.calendar.partial import MonthDay, YearMonth
from universe.time.calendar.period import Period
from universe.time.calendar.time import Time

const epoch = Date.new(1970, Month::January, 1)
const beforeEpoch = Date.new(1969, Month::December, 31)
const leapDay = Date.new(2000, Month::February, 29)
const constrained = Date.new(2024, Month::January, 31).advance(
  Period.months(1),
  onOverflow: DateOverflow::Constrain
)
const periodApplied = Date.new(2024, Month::January, 31).advance(
  Period.months(1) + Period.days(1),
  onOverflow: DateOverflow::Constrain
)
const shiftedYearMonth = YearMonth.new(2024, Month::January).plusMonths(-1)
const leapMonthDay = MonthDay.new(Month::February, 29)
const lower = Time.new(0, 0, 0, 0)
const upper = Time.new(23, 59, 59, 999_999_999)

System.print(epoch.daysSince)
System.print(beforeEpoch.daysSince)
System.print(epoch.weekday.number)
System.print(leapDay.isLeapYear)
System.print(constrained.month.number)
System.print(constrained.day)
System.print(periodApplied.month.number)
System.print(periodApplied.day)
System.print(shiftedYearMonth.year)
System.print(shiftedYearMonth.month.number)
System.print(leapMonthDay.inYear(2024).day)
System.print(lower.hour)
System.print(upper.nanosecond)
