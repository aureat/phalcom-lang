// area: bytes
// spec: bytes.md §3.1, laws 3/6; PDR-0011 rulings 3/5
// status: PASS
// fill/zeroize (one memset), slice-copies-not-views, copyInto placement and
// memmove self-copy, concat over new + copyInto_ x2 — no aliasing anywhere.

const b = Bytes.fromList([1, 2, 3, 4])
const s = b.slice(1, 3)
System.print(s.at(0))
s.set(0, 99)
System.print(b.at(1))
b.set(1, 88)
System.print(s.at(0))
const dst = Bytes.new(6)
b.copyInto(dst, 2)
System.print(dst.at(0))
System.print(dst.at(2))
System.print(dst.at(5))
const cat = Bytes.fromList([7]).concat(Bytes.fromList([8, 9]))
System.print(cat.size)
System.print(cat.at(0))
System.print(cat.at(2))
const self_copy = Bytes.fromList([5, 6])
self_copy.copyInto(self_copy, 0)
System.print(self_copy.at(1))
const f = Bytes.new(3).fill(255)
System.print(f.at(1))
System.print(f.zeroize.at(1))
