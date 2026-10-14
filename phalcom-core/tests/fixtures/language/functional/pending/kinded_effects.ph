// status: PENDING
from universe.functional import Chain, Effects, OptionContext, IdentityContext, Identity
let source = Chain<Int>::Empty.prepend(3).prepend(2).prepend(1)
let mut calls = 0
let optional = Effects.traverse<Option, OptionContext, Chain<Int>, Int, Int>(source, using: |n| {
  calls = calls + 1
  if n == 2 { Option<Int>::None } else { Option<Int>::Some(n) }
})
System.print(optional.isNone)
System.print(calls)
let identity = Effects.traverse<Identity, IdentityContext, Chain<Int>, Int, Int>(source, using: |n| { Identity<Int>(n * 2) })
System.print(identity.value.fold(initial: 0, using: |a, b| { a + b }))
let lifted = Effects.lift2<Option, OptionContext, Int, Int, Int>(Option<Int>::Some(2), Option<Int>::Some(3), using: |a, b| { a + b })
System.print(lifted.unwrap)
let nested = Option<Option<Int>>::Some(Option<Int>::Some(9))
System.print(Effects.flatten<Option, OptionContext, Int>(nested).unwrap)
