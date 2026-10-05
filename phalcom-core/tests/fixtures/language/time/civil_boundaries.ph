// area: time
// spec: stdlib/time.md
// status: PASS

from universe.time.calendar.date import Date, DateOverflow
from universe.time.calendar.month import Month
from universe.time.calendar.partial import MonthDay, YearMonth
from universe.time.calendar.period import Period
from universe.time.calendar.time import Time

let epoch = Date.new(1970, Month::January, 1)
let beforeEpoch = Date.new(1969, Month::December, 31)
let leapDay = Date.new(2000, Month::February, 29)
let constrained = Date.new(2024, Month::January, 31).advance(
  Period.months(1),
  onOverflow: DateOverflow::Constrain
)
let periodApplied = Date.new(2024, Month::January, 31).advance(
  Period.months(1) + Period.days(1),
  onOverflow: DateOverflow::Constrain
)
let shiftedYearMonth = YearMonth.new(2024, Month::January).plusMonths(-1)
let leapMonthDay = MonthDay.new(Month::February, 29)
let lower = Time.new(0, 0, 0, 0)
let upper = Time.new(23, 59, 59, 999_999_999)

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
