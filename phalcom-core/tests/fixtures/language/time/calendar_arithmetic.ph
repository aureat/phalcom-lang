// area: time
// spec: stdlib/time.md
// status: PASS

from universe.time.calendar.month import Gregorian

System.print(Gregorian.isLeapYear(2000))
System.print(Gregorian.isLeapYear(1900))
System.print(Gregorian.isLeapYear(2024))
System.print(Gregorian.daysFromCivil(1970, 1, 1))
System.print(Gregorian.daysFromCivil(1969, 12, 31))
System.print(Gregorian.daysFromCivil(2000, 2, 29))

let epoch = Gregorian.civilFromDays(0)
System.print(epoch.get(#year).unwrapOr(-1))
System.print(epoch.get(#month).unwrapOr(-1))
System.print(epoch.get(#day).unwrapOr(-1))

let beforeEpoch = Gregorian.civilFromDays(-1)
System.print(beforeEpoch.get(#year).unwrapOr(-1))
System.print(beforeEpoch.get(#month).unwrapOr(-1))
System.print(beforeEpoch.get(#day).unwrapOr(-1))

let yearZeroDays = Gregorian.daysFromCivil(0, 1, 1)
System.print(yearZeroDays)
let yearZero = Gregorian.civilFromDays(yearZeroDays)
System.print(yearZero.get(#year).unwrapOr(-1))
System.print(yearZero.get(#month).unwrapOr(-1))
System.print(yearZero.get(#day).unwrapOr(-1))

System.print(Gregorian.addMonthsYear(2025, 1, -1))
System.print(Gregorian.addMonthsNumber(2025, 1, -1))
System.print(Gregorian.addMonthsYear(2024, 2, 12))
System.print(Gregorian.addMonthsNumber(2024, 2, 12))
