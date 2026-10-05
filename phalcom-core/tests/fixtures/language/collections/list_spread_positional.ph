// area: collections
// spec: collection spread Part I §§2.2–2.4, 9–11
// status: PASS
// List spread consumes Unit as empty, Tuple positionals without its labeled
// lane, and ordinary cursor-protocol sources in lexical order.

let mixed = [0, *(1, 2, name: 3), 4]
let fromList = [*[5, 6], 7]
let fromRange = [*(0..3)]
let fromUnit = [*(), 8]
let limited = [*(0..).iter.take(3)]
let gcList = [[1, 2], *(), System.gc]
let large = [*(0..=65535)]

System.print(mixed.toString)        // [0, 1, 2, 4]
System.print(fromList.toString)     // [5, 6, 7]
System.print(fromRange.toString)    // [0, 1, 2]
System.print(fromUnit.toString)     // [8]
System.print(limited.toString)      // [0, 1, 2]
System.print(gcList.toString)       // [[1, 2], ()]
System.print(large.size)            // 65536
