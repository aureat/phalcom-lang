// High fiber-turnover probe: spawn -> run to Done -> drop, 500k times.
// Every iteration hits the pool's only recycle site (dispatch.rs Done path),
// which skynet (spawn-once, never respawn) does not exercise.
// Checksum: sum of 0..499999 = 124999750000.
var i = 0
var acc = 0
while (i < 500000) {
  var f = Fiber.new { i }
  acc = acc + f.call()
  i = i + 1
}
System.print(acc)
