// area: bytes
// spec: bytes.md §3-§5 (laws 1-4); PDR-0011
// status: PASS
// Zero-filled birth, size stability, set/at roundtrip, at-totality (bare
// octet or None, never a raise), debug toString.

const b = Bytes.new(4)
System.print(b.toString)
System.print(b.size)
System.print(b.at(3))
b.set(0, 72).set(1, 105)
System.print(b.at(0))
System.print(b.at(1))
System.print(b.at(9))
b[2] = 33
System.print(b[2])
System.print(b.size)
