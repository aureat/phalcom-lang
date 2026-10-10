from universe.random import Xoshiro256PlusPlus

let rng = Xoshiro256PlusPlus.seed(42)
let mut count = 0
let mut index = 0
while index < 1000000 {
  let value = rng.nextFloat
  if value >= 0.0 and value < 1.0 { count = count + 1 }
  index = index + 1
}
System.print(count == 1000000)
