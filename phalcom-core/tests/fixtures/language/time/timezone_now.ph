// area: time
// status: PASS

from universe.time import Date, TimeZone, UtcOffset, ZonedDateTime

let zone = TimeZone.fixed(UtcOffset.seconds(0))
System.print(Date.todayIn(zone).is(Date))
System.print(ZonedDateTime.nowIn(zone).is(ZonedDateTime))
System.print(ZonedDateTime.now.is(ZonedDateTime))
