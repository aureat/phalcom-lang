// A1-A3 product regression: Record equality and hash ignore encounter order.

const first = #{a: 1, b: 2}
const second = #{b: 2, a: 1}

System.print(first == second)
System.print(first.hash == second.hash)
