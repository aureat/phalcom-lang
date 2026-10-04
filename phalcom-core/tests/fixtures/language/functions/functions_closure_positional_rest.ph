// area: functions
// spec: callable surface / parameter foundations
// status: PASS

let collect = |head, *tail| tail
let wide = |a, b, c, d, e| e
System.print(collect.call(1, 2, 3).size)
System.print(collect.call(1).class == Unit)
System.print(collect.callWith((1, 2, 3)).size)
System.print(wide.call(1, 2, 3, 4, 5))
