from universe.random import Xoshiro256PlusPlus

let rng = Xoshiro256PlusPlus.seed(42)
let mut sum = 0
let mut index = 0
while index < 100000 {
  sum = sum + rng.nextInt(below: 1 << 128)
  index = index + 1
}
System.print(sum >= 0)
