from universe.random import Xoshiro256PlusPlus

let bytes = Xoshiro256PlusPlus.seed(42).bytes(1024)
let mut checksum = 0
let mut index = 0
while index < bytes.size {
  checksum = checksum + bytes.at(index)
  index = index + 1
}
System.print(bytes.size == 1024 and checksum >= 0)
