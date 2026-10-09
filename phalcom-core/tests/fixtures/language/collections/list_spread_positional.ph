// area: collections
// spec: collection spread Part I §§2.2–2.4, 9–11
// status: PASS
// List spread consumes Unit as empty, Tuple positionals without its labeled
// lane, and ordinary cursor-protocol sources in lexical order.

let mixed: List<Int> = [0, *(1, 2, name: 3), 4]
let spreadSource: List<Int> = [5, 6]
let fromList: List<Int> = [*spreadSource, 7]
let fromRange: List<Int> = [*(0..3)]
let fromUnit: List<Int> = [*(), 8]
let limited: List<Int> = [*(0..).iter.take(3)]
let nested: List<Int> = [1, 2]
let gcList: List<Dynamic> = [nested, *(), System.gc]
let large: List<Int> = [*(0..=65535)]

System.print(mixed.toString)        // [0, 1, 2, 4]
System.print(fromList.toString)     // [5, 6, 7]
System.print(fromRange.toString)    // [0, 1, 2]
System.print(fromUnit.toString)     // [8]
System.print(limited.toString)      // [0, 1, 2]
System.print(gcList.toString)       // [[1, 2], ()]
System.print(large.size)            // 65536
