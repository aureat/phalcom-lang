// area: time
// spec: stdlib/time.md
// status: PASS

from universe.time import UtcOffset

let zero = UtcOffset.utc
let positive = UtcOffset.seconds(1)
let negative = UtcOffset.seconds(-1)
let lower = UtcOffset.seconds(-93_599)
let upper = UtcOffset.seconds(93_599)
let historical = UtcOffset.seconds(-21_036)
let hourOffset = UtcOffset.hours(25)

System.print(zero.seconds)
System.print(positive.seconds)
System.print(negative.seconds)
System.print(lower.seconds)
System.print(upper.seconds)
System.print(historical.seconds)
System.print(hourOffset.seconds)
System.print(lower.negated.seconds)
System.print(upper.negated.seconds)
System.print(lower.negated.negated == lower)
System.print(lower < zero)
System.print(zero < upper)
System.print(historical == UtcOffset.seconds(-21_036))
System.print(historical.hash == UtcOffset.seconds(-21_036).hash)
