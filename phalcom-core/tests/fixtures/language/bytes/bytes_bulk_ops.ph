// area: bytes
// spec: bytes.md §3.1, laws 3/6; PDR-0011 rulings 3/5
// status: PASS
// fill/zeroize (one memset), slice-copies-not-views, copyInto placement and
// memmove self-copy, concat over new + copyInto_ x2 — no aliasing anywhere.

let initialValues: List<Int> = [1, 2, 3, 4]
let b = Bytes.fromList(initialValues)
let s = b.slice(1, 3)
System.print(s.at(0))
s.set(0, 99)
System.print(b.at(1))
b.set(1, 88)
System.print(s.at(0))
let dst = Bytes.new(6)
b.copyInto(dst, 2)
System.print(dst.at(0))
System.print(dst.at(2))
System.print(dst.at(5))
let leftValues: List<Int> = [7]
let rightValues: List<Int> = [8, 9]
let cat = Bytes.fromList(leftValues).concat(Bytes.fromList(rightValues))
System.print(cat.size)
System.print(cat.at(0))
System.print(cat.at(2))
let selfCopyValues: List<Int> = [5, 6]
let self_copy = Bytes.fromList(selfCopyValues)
self_copy.copyInto(self_copy, 0)
System.print(self_copy.at(1))
let f = Bytes.new(3).fill(255)
System.print(f.at(1))
System.print(f.zeroize.at(1))
