// area: math
// status: PASS

import universe.math as math

let hugeA = 123456789012345678901234567890
let hugeB = 987654321098765432109876543210
let root100 = math.isqrt(10 ** 100)
let root1000 = math.isqrt(10 ** 1000)
let k = 10 ** 80 + 123
let square = k * k

System.print(math.gcd(0, 0))
System.print(math.gcd(0, 1))
System.print(math.gcd(84, 30))
System.print(math.gcd(-84, 30))
System.print(math.gcd(84, -30))
System.print(math.gcd(-84, -30))
System.print(math.lcm(0, hugeA))
System.print(math.lcm(-21, 6))
System.print(math.gcd(hugeA, hugeB))
System.print(math.lcm(hugeA, hugeB))
System.print(math.isqrt(0))
System.print(math.isqrt(1))
System.print(math.isqrt(2))
System.print(math.isqrt(3))
System.print(math.isqrt(4))
System.print(math.isqrt(square - 1))
System.print(math.isqrt(square))
System.print(math.isqrt(square + 1))
System.print(root100)
System.print(root1000)
System.print(math.isqrt(1 << 4096).bitLength)
