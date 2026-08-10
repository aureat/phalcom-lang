// area: collections
// spec: collection spread Part I §§2.2–2.4, 9–11
// status: PASS
// List spread consumes Unit as empty, Tuple positionals without its labeled
// lane, and ordinary cursor-protocol sources in lexical order.

const mixed = [0, *(1, 2, name: 3), 4]
const fromList = [*[5, 6], 7]
const fromRange = [*(0..3)]
const fromUnit = [*(), 8]
const limited = [*(0..).iter.take(3)]

System.print(mixed.toString)
System.print(fromList.toString)
System.print(fromRange.toString)
System.print(fromUnit.toString)
System.print(limited.toString)
