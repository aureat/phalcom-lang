from universe.random import Xoshiro256PlusPlus

let values = List<Int>.new()
let mut index = 0
while index < 100 {
  values.append(index)
  index = index + 1
}
Xoshiro256PlusPlus.seed(42).shuffle(values)
let mut sum = 0
index = 0
while index < values.size {
  sum = sum + values.at(index)
  index = index + 1
}
System.print(values.size == 100 and sum == 4950)
