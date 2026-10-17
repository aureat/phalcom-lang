// status: PENDING
from universe.functional import Iso, Prism
data Wrapper(value: Int)
let iso = Iso<Wrapper, Int>.new(forward: |source| { source.value }, backward: |n| { Wrapper(value: n) })
System.print(iso.convert(Wrapper(value: 4)))
System.print(iso.inverse.convert(5).value)
System.print(iso.asLens.modify(Wrapper(value: 4), using: |n| { n + 3 }).value)
let some = Prism<Option<Int>, Int>.new(
  preview: |source| { source },
  review: |value| { Option<Int>::Some(value) }
)
let mut calls = 0
let absent = some.modify(Option<Int>::None, using: |n| { calls = calls + 1; n + 1 })
System.print(absent.isNone)
System.print(calls)
System.print(some.asOptional.set(Option<Int>::Some(2), to: 8).unwrap)
System.print(some.asOptional.set(Option<Int>::None, to: 8).isNone)
// The reflected shape guard is exercised with an exact same-type product.
let shaped = Iso<Wrapper, Wrapper>.sameShape(forward: |s| { s }, backward: |s| { s })
System.print(shaped.convert(Wrapper(value: 9)).value)
