class Minstd {
  mut _state: Int

  @constructor
  new(_ seed: Int) { _state = seed }

  next -> Int {
    _state = (16807 * _state) % 2147483647
    _state
  }
}

let rng = Minstd.new(42)
let mut sum = 0
let mut index = 0
while index < 1000000 {
  sum = sum + rng.next
  index = index + 1
}
System.print(sum > 0)
