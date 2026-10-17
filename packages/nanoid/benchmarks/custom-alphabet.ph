from nanoid.alphabet import Alphabet
from nanoid.generator import NanoIdGenerator

let iterations = 1000

// Power-of-two (size 16)
let hexAlpha = Alphabet.new("0123456789abcdef")
let hexGen = NanoIdGenerator.new(hexAlpha, 16)

System.print("Benchmarking power-of-two alphabet (size 16) across \(iterations) iterations...")
for i in 0..iterations {
  let id = hexGen.generate()
}

// Rejection sampling (size 62)
let base62Alpha = Alphabet.new("0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz")
let base62Gen = NanoIdGenerator.new(base62Alpha, 21)

System.print("Benchmarking non-power-of-two alphabet (size 62) across \(iterations) iterations...")
for j in 0..iterations {
  let id = base62Gen.generate()
}

System.print("Custom alphabet benchmarks complete.")
