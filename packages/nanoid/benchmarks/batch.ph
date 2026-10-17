from nanoid.id import NanoId

let sizes = [12, 64, 128, 1024, 4096]
let iterations = 100

System.print("Benchmarking batch scaling across sizes: 12, 64, 128, 1024, 4096...")

for size in sizes {
  let mut i = 0
  for i in 0..iterations {
    let id = NanoId.generate(size)
  }
  System.print("Completed size \(size) x \(iterations) iterations.")
}

System.print("Batch scaling benchmark complete.")
