from measure.standard.si.base import m
let mut distance = m(0)
let step = m(1)
let mut i = 0
while i < 100000 {
  distance = distance + step
  i = i + 1
}
System.print(distance.toString)
