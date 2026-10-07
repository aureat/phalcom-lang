// area: math
// status: PASS

from universe.math import pi, tau, sqrt, hypot, gcd, lcm, isqrt
import universe.math as math

System.print(pi)
System.print(tau)
System.print(math.e)
System.print(sqrt(9.0))
System.print(hypot(3.0, 4.0))
System.print(math.log(8.0, base: 2.0))
System.print(math.degrees(pi))
System.print(math.radians(180.0))
System.print(gcd(-84, 30))
System.print(lcm(-21, 6))
System.print(isqrt(10_000))
