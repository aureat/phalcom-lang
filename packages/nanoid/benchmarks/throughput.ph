from nanoid.id import NanoId
from nanoid.generator import NanoIdGenerator

let total = 5000
let mut i = 0

System.print("Measuring sustained generation throughput across \(total) IDs...")
let gen = NanoIdGenerator.default

while i < total {
  let id = gen.generate()
  i = i + 1
}

System.print("Throughput benchmark completed \(total) IDs.")
