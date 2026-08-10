// area: functions
// spec: callable surface / parameter foundations
// status: PASS

const collect = |head, *tail| tail
System.print(collect.call(1, 2, 3).size)
System.print(collect.call(1).size)
