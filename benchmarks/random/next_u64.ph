from universe.random import Xoshiro256PlusPlus

let rng = Xoshiro256PlusPlus.seed(42)
let mut checksum = 0
let mut index = 0
while index < 1000000 {
  checksum = checksum ^ rng.nextU64
  index = index + 1
}
System.print(checksum >= 0)
