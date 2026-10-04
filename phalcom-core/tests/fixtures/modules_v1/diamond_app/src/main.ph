from .a import ServiceA
from .b import ServiceB

let mut a = ServiceA.new()
let mut b = ServiceB.new()
if (a.config != 42) {
  throw Error.new("Assertion failed: diamond imports do not match")
}
if (b.config != 42) {
  throw Error.new("Assertion failed: diamond imports do not match")
}
