// area: time
// spec: stdlib/time.md
// status: PASS

from universe.time.calendar.date import Date
from universe.time.calendar.datetime import DateTime
from universe.time.calendar.month import Month
from universe.time.calendar.partial import MonthDay, YearMonth
from universe.time.calendar.time import Time

let date = Date.new(2024, Month::February, 29)
let time = Time.new(23, 59, 58, 123)
let datetime = DateTime.new(date, time)
let yearMonth = YearMonth.new(2024, Month::February)
let monthDay = MonthDay.new(Month::February, 29)

System.print(date.dayOfYear)
System.print(date.weekday.number)
System.print(time.hour)
System.print(time.nanosecond)
System.print(datetime.date.day)
System.print(datetime.time.second)
System.print(yearMonth.days)
System.print(yearMonth.isLeapYear)
System.print(monthDay.inYear(2024).day)
