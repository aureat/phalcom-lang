from nanoid.id import NanoId

System.print("Benchmarking default NanoId.generate() across \(iterations) iterations...")
for i in 0..1000 {
  let id = NanoId.generate()
}
System.print("Default benchmark complete.")
